use crate::decode::{decode_stored_messages, received_pdus, StoredPdu};

const SINGLE: &str = "0000049121430008620102214365000400480069";
const FIRST: &str = "0040049121430008620102214365000A0500032A020100480069";
const SECOND: &str = "0040049121430008620102214365000A0500032A020200210021";

fn entry(index: u32, pdu: &str) -> StoredPdu {
    StoredPdu {
        index,
        pdu: pdu.to_string(),
    }
}

#[test]
fn cmgl_only_authorizes_received_records() {
    let response = format!(
        "+CMGL: 1,0,,20\r\n{SINGLE}\r\n+CMGL: 2,1,,20\r\n{SINGLE}\r\n\
         +CMGL: 3,2,,20\r\n{SINGLE}\r\n+CMGL: 4,3,,20\r\n{SINGLE}\r\nOK\r\n"
    );
    let entries = received_pdus(&response);
    assert_eq!(
        entries.iter().map(|entry| entry.index).collect::<Vec<_>>(),
        vec![1, 2]
    );
}

#[test]
fn every_truncated_pdu_is_preserved_without_panicking() {
    for length in 0..SINGLE.len() {
        assert!(decode_stored_messages(vec![entry(1, &SINGLE[..length])], "sim").is_empty());
    }
    for pdu in [
        SINGLE.replace("620102", "623102"),
        SINGLE.replace("912143", "9121A3"),
        SINGLE.replacen("000004", "000104", 1),
        SINGLE.replacen("000862", "00F562", 1),
        SINGLE.replace("00480069", "D8000069"),
    ] {
        assert!(decode_stored_messages(vec![entry(1, &pdu)], "sim").is_empty());
    }
}

#[test]
fn multipart_references_are_isolated_by_sender_and_keep_all_indexes() {
    let other_first = FIRST
        .replace("912143", "916587")
        .replace("00480069", "00410042");
    let other_second = SECOND
        .replace("912143", "916587")
        .replace("00210021", "00430044");
    let mut messages = decode_stored_messages(
        vec![
            entry(4, &other_second),
            entry(2, SECOND),
            entry(1, FIRST),
            entry(3, &other_first),
        ],
        "sim",
    );
    messages.sort_by(|a, b| a.sms.contact.cmp(&b.sms.contact));
    assert_eq!(messages.len(), 2);
    assert_eq!(messages[0].sms.message, "Hi!!");
    assert_eq!(messages[1].sms.message, "ABCD");
    assert_eq!(
        messages[0]
            .parts
            .iter()
            .map(|part| part.index)
            .collect::<Vec<_>>(),
        vec![1, 2]
    );
    assert_eq!(
        messages[1]
            .parts
            .iter()
            .map(|part| part.index)
            .collect::<Vec<_>>(),
        vec![3, 4]
    );
}

#[test]
fn conflicting_multipart_sequences_are_preserved() {
    let conflicting = FIRST.replace("00480069", "00420079");
    assert!(decode_stored_messages(
        vec![entry(1, FIRST), entry(2, &conflicting), entry(3, SECOND)],
        "sim"
    )
    .is_empty());
}

#[test]
fn sixteen_bit_multipart_reference_is_supported() {
    let messages = decode_stored_messages(
        vec![
            entry(1, "0040049121430008620102214365000B0608041234020100480069"),
            entry(2, "0040049121430008620102214365000B0608041234020200210021"),
        ],
        "sim",
    );
    assert_eq!(messages.len(), 1);
    assert_eq!(messages[0].sms.message, "Hi!!");
}

#[test]
fn gsm_septet_length_and_extension_table_are_respected() {
    let messages = decode_stored_messages(
        vec![
            entry(1, "00000491214300006201022143650005E8329BFD06"),
            entry(2, "000004912143000062010221436500021B0A"),
        ],
        "sim",
    );
    assert_eq!(messages[0].sms.message, "hello");
    assert_eq!(messages[1].sms.message, "^");
}

#[test]
fn utf16_surrogate_pairs_are_archived_without_replacement_characters() {
    let messages = decode_stored_messages(
        vec![entry(1, &SINGLE.replace("00480069", "D83DDE00"))],
        "sim",
    );
    assert_eq!(messages[0].sms.message, "😀");
}

#[test]
fn gsm_multipart_padding_does_not_corrupt_the_archived_text() {
    let messages = decode_stored_messages(
        vec![
            entry(1, "0040049121430000620102214365000C0500032A0201D06536FB0D"),
            entry(2, "0040049121430000620102214365000C0500032A0202D06536FB0D"),
        ],
        "sim",
    );
    assert_eq!(messages.len(), 1);
    assert_eq!(messages[0].sms.message, "hellohello");
}

#[test]
fn eight_and_sixteen_bit_references_do_not_share_a_multipart_group() {
    let messages = decode_stored_messages(
        vec![
            entry(1, FIRST),
            entry(2, "0040049121430008620102214365000B060804002A020200210021"),
        ],
        "sim",
    );
    assert!(messages.is_empty());
}
