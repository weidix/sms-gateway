import { writable } from 'svelte/store';
import { getStorageValue, updateStorageValue } from '../js/storage';
import { apiClient } from '../js/api';

// 认证状态管理
export const isAuthenticated = writable(false);
export const isAuthLoading = writable(true);
export const isLoginRequired = writable(true);

// 立即开始初始化认证状态
initAuth();

// 初始化认证状态
export async function initAuth() {
    isAuthLoading.set(true);
    
    try {
        const authStatus = await apiClient.getAuthStatus();
        const loginRequired = authStatus?.login_required !== false;
        isLoginRequired.set(loginRequired);

        if (!loginRequired) {
            isAuthenticated.set(true);
            return;
        }

        const authData = await getStorageValue('auth');
        const isValid = authData && await apiClient.checkAuth();
        
        isAuthenticated.set(!!isValid);
    } catch (error) {
        console.error('Auth initialization failed:', error);
        isAuthenticated.set(false);
    } finally {
        isAuthLoading.set(false);
    }
}

// 登出
export async function logout() {
    try {
        await updateStorageValue('auth', null);
        isAuthenticated.set(false);
        window.location.reload();
    } catch (error) {
        console.error('Logout failed:', error);
    }
}
