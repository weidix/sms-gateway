use chrono::NaiveDateTime;
use std::collections::HashMap;

use crate::db::ModemSMS;

/// A received PDU and its current index in the selected modem storage.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct StoredPdu {
    pub index: u32,
    pub pdu: String,
}

/// A complete SMS together with every device record needed to reconstruct it.
#[derive(Debug)]
pub(crate) struct IndexedSms {
    pub sms: ModemSMS,
    pub parts: Vec<StoredPdu>,
}

#[derive(Debug, PartialEq, Eq, Hash)]
enum MultipartReference {
    EightBit(u8),
    SixteenBit(u16),
}

type MultipartKey = (String, MultipartReference, u8);

struct PendingMultipart {
    timestamp: NaiveDateTime,
    parts: Vec<Option<(String, StoredPdu)>>,
    conflicting: bool,
}

/// Extracts only received records; sent messages and drafts are left on the device.
pub(crate) fn received_pdus(response: &str) -> Vec<StoredPdu> {
    let mut lines = response.lines().map(str::trim);
    let mut entries = Vec::new();
    while let Some(line) = lines.next() {
        let Some(header) = line.strip_prefix("+CMGL:") else {
            continue;
        };
        let mut fields = header.split(',').map(str::trim);
        let Some(index) = fields.next().and_then(|value| value.parse().ok()) else {
            continue;
        };
        if !matches!(fields.next(), Some("0" | "1")) {
            continue;
        }
        if let Some(pdu) = lines.next().filter(|pdu| !pdu.is_empty()) {
            entries.push(StoredPdu {
                index,
                pdu: pdu.to_ascii_uppercase(),
            });
        }
    }
    entries
}

/// Parses complete received messages without discarding their modem indexes.
/// Invalid PDUs and incomplete or conflicting multipart groups remain on the device.
pub(crate) fn decode_stored_messages(entries: Vec<StoredPdu>, sim_id: &str) -> Vec<IndexedSms> {
    let mut pending: HashMap<MultipartKey, PendingMultipart> = HashMap::new();
    let mut messages = Vec::new();
    for entry in entries {
        let Some((sender, timestamp, content)) = decode_deliver_pdu(&entry.pdu) else {
            log::warn!("Leaving undecodable SMS at device index {}", entry.index);
            continue;
        };
        match content {
            MessageContent::Single(message) => messages.push(IndexedSms {
                sms: ModemSMS {
                    contact: sender,
                    timestamp,
                    message,
                    send: false,
                    sim_id: sim_id.to_string(),
                },
                parts: vec![entry],
            }),
            MessageContent::Multipart {
                reference,
                total,
                current,
                content,
            } => {
                if current == 0 || current > total {
                    continue;
                }
                let group = pending
                    .entry((sender, reference, total))
                    .or_insert_with(|| PendingMultipart {
                        timestamp,
                        parts: vec![None; usize::from(total)],
                        conflicting: false,
                    });
                let slot = &mut group.parts[usize::from(current - 1)];
                if slot.is_some() {
                    // Ambiguous reference reuse must never authorize deletion.
                    group.conflicting = true;
                } else {
                    *slot = Some((content, entry));
                }
            }
        }
    }
    for ((sender, _, _), group) in pending {
        if group.conflicting || group.parts.iter().any(Option::is_none) {
            continue;
        }
        let mut message = String::new();
        let mut parts = Vec::new();
        for (content, entry) in group.parts.into_iter().flatten() {
            message.push_str(&content);
            parts.push(entry);
        }
        messages.push(IndexedSms {
            sms: ModemSMS {
                contact: sender,
                timestamp: group.timestamp,
                message,
                send: false,
                sim_id: sim_id.to_string(),
            },
            parts,
        });
    }
    messages
}

/// Decodes received messages for read-only callers without authorizing cleanup.
pub fn parse_pdu_sms(cmgl_entries: &str, sim_id: &str) -> Vec<ModemSMS> {
    decode_stored_messages(received_pdus(cmgl_entries), sim_id)
        .into_iter()
        .map(|message| message.sms)
        .collect()
}

enum MessageContent {
    Multipart {
        reference: MultipartReference,
        total: u8,
        current: u8,
        content: String,
    },
    Single(String),
}

fn take_bytes<'a>(bytes: &mut &'a [u8], count: usize) -> Option<&'a [u8]> {
    let result = bytes.get(..count)?;
    *bytes = bytes.get(count..)?;
    Some(result)
}

fn take_byte(bytes: &mut &[u8]) -> Option<u8> {
    Some(*take_bytes(bytes, 1)?.first()?)
}

fn decode_deliver_pdu(pdu: &str) -> Option<(String, NaiveDateTime, MessageContent)> {
    let decoded = hex::decode(pdu).ok()?;
    let mut bytes = decoded.as_slice();
    let smsc_length = usize::from(take_byte(&mut bytes)?);
    take_bytes(&mut bytes, smsc_length)?;
    let pdu_type = take_byte(&mut bytes)?;
    if pdu_type & 0x03 != 0 {
        return None;
    }
    let sender_length = usize::from(take_byte(&mut bytes)?);
    if sender_length == 0 || sender_length > 20 {
        return None;
    }
    let sender_type = take_byte(&mut bytes)?;
    let sender_bytes = take_bytes(&mut bytes, sender_length.div_ceil(2))?;
    let sender = if sender_type & 0x70 == 0x50 {
        decode_gsm7bit_with_offset(sender_bytes, 0, sender_length * 4 / 7)?
    } else {
        let number = decode_bcd(sender_bytes, sender_length)?;
        if sender_type & 0x70 == 0x10 {
            format!("+{number}")
        } else {
            number
        }
    };
    take_byte(&mut bytes)?; // Protocol identifier
    let dcs = take_byte(&mut bytes)?;
    // Unsupported encodings must not be archived as corrupted text and deleted.
    if !matches!(dcs, 0x00 | 0x08) {
        return None;
    }
    let timestamp = parse_timestamp(take_bytes(&mut bytes, 7)?)?;
    let length = usize::from(take_byte(&mut bytes)?);
    let octets = if dcs == 0 {
        (length * 7).div_ceil(8)
    } else {
        length
    };
    let data = take_bytes(&mut bytes, octets)?;
    let content = parse_message_content(data, dcs, length, pdu_type & 0x40 != 0)?;
    Some((sender, timestamp, content))
}

fn parse_message_content(
    bytes: &[u8],
    dcs: u8,
    length: usize,
    has_udhi: bool,
) -> Option<MessageContent> {
    if !has_udhi {
        return decode_user_data(bytes, dcs, length, 0).map(MessageContent::Single);
    }
    let header_length = usize::from(*bytes.first()?) + 1;
    let mut header = bytes.get(1..header_length)?;
    let mut multipart = None;
    while !header.is_empty() {
        let kind = take_byte(&mut header)?;
        let size = usize::from(take_byte(&mut header)?);
        let value = take_bytes(&mut header, size)?;
        match (kind, value) {
            (0x00, [reference, total, current]) if multipart.is_none() => {
                multipart = Some((MultipartReference::EightBit(*reference), *total, *current))
            }
            (0x08, [high, low, total, current]) if multipart.is_none() => {
                multipart = Some((
                    MultipartReference::SixteenBit(u16::from_be_bytes([*high, *low])),
                    *total,
                    *current,
                ))
            }
            // Leave port-addressed and other unsupported user data untouched.
            _ => return None,
        }
    }
    let (reference, total, current) = multipart?;
    let content = decode_user_data(bytes, dcs, length, header_length)?;
    Some(MessageContent::Multipart {
        reference,
        total,
        current,
        content,
    })
}

fn decode_user_data(bytes: &[u8], dcs: u8, length: usize, header_length: usize) -> Option<String> {
    if dcs == 0 {
        let header_septets = (header_length * 8).div_ceil(7);
        decode_gsm7bit_with_offset(
            bytes,
            header_septets * 7,
            length.checked_sub(header_septets)?,
        )
    } else {
        let data = bytes.get(header_length..)?;
        if dcs == 0x08 && data.len() % 2 != 0 {
            return None;
        }
        decode_ucs2(data)
    }
}

fn decode_gsm7bit_with_offset(bytes: &[u8], bit_offset: usize, septets: usize) -> Option<String> {
    let mut result = String::new();
    let mut escaped = false;
    for index in 0..septets {
        let bit = bit_offset + index * 7;
        let mut value = u16::from(*bytes.get(bit / 8)?) >> (bit % 8);
        if bit % 8 > 1 {
            value |= u16::from(*bytes.get(bit / 8 + 1)?) << (8 - bit % 8);
        }
        let septet = (value & 0x7f) as u8;
        if escaped {
            result.push(match septet {
                0x0a => '\u{000c}',
                0x14 => '^',
                0x28 => '{',
                0x29 => '}',
                0x2f => '\\',
                0x3c => '[',
                0x3d => '~',
                0x3e => ']',
                0x40 => '|',
                0x65 => '€',
                _ => return None,
            });
            escaped = false;
        } else if septet == 0x1b {
            escaped = true;
        } else {
            result.push(gsm7bit_to_char(septet));
        }
    }
    (!escaped).then_some(result)
}

fn gsm7bit_to_char(septet: u8) -> char {
    match septet {
        0x00 => '@',
        0x01 => '£',
        0x02 => '$',
        0x03 => '¥',
        0x04 => 'è',
        0x05 => 'é',
        0x06 => 'ù',
        0x07 => 'ì',
        0x08 => 'ò',
        0x09 => 'Ç',
        0x0A => '\n',
        0x0B => 'Ø',
        0x0C => 'ø',
        0x0D => '\r',
        0x0E => 'Å',
        0x0F => 'å',
        0x10 => '\u{0394}', // Δ
        0x11 => '_',
        0x12 => '\u{03A6}', // Φ
        0x13 => '\u{0393}', // Γ
        0x14 => '\u{039B}', // Λ
        0x15 => '\u{03A9}', // Ω
        0x16 => '\u{03A0}', // Π
        0x17 => '\u{03A8}', // Ψ
        0x18 => '\u{03A3}', // Σ
        0x19 => '\u{0398}', // Θ
        0x1A => '\u{039E}', // Ξ
        0x1B => '\u{001B}', // escape for extended table
        0x1C => 'Æ',
        0x1D => 'æ',
        0x1E => 'ß',
        0x1F => 'É',
        0x20 => ' ',
        0x24 => '¤',
        0x40 => '¡',
        0x5b => 'Ä',
        0x5c => 'Ö',
        0x5d => 'Ñ',
        0x5e => 'Ü',
        0x5f => '§',
        0x60 => '¿',
        0x7b => 'ä',
        0x7c => 'ö',
        0x7d => 'ñ',
        0x7e => 'ü',
        0x7f => 'à',
        0x21..=0x7F => septet as char,
        _ => '?',
    }
}

fn decode_ucs2(bytes: &[u8]) -> Option<String> {
    let units: Vec<u16> = bytes
        .chunks_exact(2)
        .map(|pair| u16::from_be_bytes([pair[0], pair[1]]))
        .collect();
    String::from_utf16(&units).ok()
}

fn decode_bcd(bytes: &[u8], len: usize) -> Option<String> {
    bytes
        .iter()
        .flat_map(|b| [b & 0x0F, (b >> 4) & 0x0F].into_iter())
        .take(len)
        .map(|n| (n <= 9).then_some((n + b'0') as char))
        .collect()
}

fn parse_timestamp(bytes: &[u8]) -> Option<NaiveDateTime> {
    let decode = |b: u8| {
        let high = b >> 4;
        let low = b & 0x0f;
        (high <= 9 && low <= 9).then_some(u32::from(low) * 10 + u32::from(high))
    };
    chrono::NaiveDate::from_ymd_opt(
        2000 + i32::try_from(decode(*bytes.first()?)?).ok()?,
        decode(*bytes.get(1)?)?,
        decode(*bytes.get(2)?)?,
    )?
    .and_hms_opt(
        decode(*bytes.get(3)?)?,
        decode(*bytes.get(4)?)?,
        decode(*bytes.get(5)?)?,
    )
}
