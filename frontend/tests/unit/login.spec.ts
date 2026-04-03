import { describe, it, expect, vi, beforeEach } from 'vitest';
import { mount, flushPromises } from '@vue/test-utils';
import { createTestingPinia } from '@pinia/testing';
import Login from '@/views/Login.vue';
import { useAccountStore } from '@/stores/account';

// Mock axios
vi.mock('axios', () => ({
  default: {
    post: vi.fn(),
  },
}));

import axios from 'axios';

describe('Login Component', () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  it('renders login form correctly', () => {
    const wrapper = mount(Login, {
      global: {
        plugins: [createTestingPinia()],
      },
    });

    expect(wrapper.find('input[name="username"]').exists()).toBe(true);
    expect(wrapper.find('input[name="password"]').exists()).toBe(true);
    expect(wrapper.find('button[type="submit"]').exists()).toBe(true);
  });

  it('submits login form with correct credentials', async () => {
    const mockResponse = { data: { token: 'mock-jwt-token', user: { id: 1, username: 'test' } } };
    (axios.post as any).mockResolvedValue(mockResponse);

    const wrapper = mount(Login, {
      global: {
        plugins: [createTestingPinia()],
      },
    });

    await wrapper.find('input[name="username"]').setValue('testuser');
    await wrapper.find('input[name="password"]').setValue('password123');
    await wrapper.find('form').trigger('submit.prevent');

    await flushPromises();

    expect(axios.post).toHaveBeenCalledWith(
      '/api/v1/auth/login',
      { username: 'testuser', password: 'password123' }
    );
  });

  it('handles login success and stores token', async () => {
    const mockResponse = { data: { token: 'mock-jwt-token', user: { id: 1, username: 'test' } } };
    (axios.post as any).mockResolvedValue(mockResponse);

    const wrapper = mount(Login, {
      global: {
        plugins: [createTestingPinia()],
      },
    });

    await wrapper.find('input[name="username"]').setValue('testuser');
    await wrapper.find('input[name="password"]').setValue('password123');
    await wrapper.find('form').trigger('submit.prevent');

    await flushPromises();

    const accountStore = useAccountStore();
    expect(accountStore.token).toBe('mock-jwt-token');
    expect(accountStore.user?.username).toBe('test');
  });

  it('handles login failure', async () => {
    const mockError = { response: { data: { message: 'Invalid credentials' } } };
    (axios.post as any).mockRejectedValue(mockError);

    const wrapper = mount(Login, {
      global: {
        plugins: [createTestingPinia()],
      },
    });

    await wrapper.find('input[name="username"]').setValue('wronguser');
    await wrapper.find('input[name="password"]').setValue('wrongpass');
    await wrapper.find('form').trigger('submit.prevent');

    await flushPromises();

    expect(wrapper.find('.error-message').exists()).toBe(true);
    expect(wrapper.find('.error-message').text()).toContain('Invalid credentials');
  });

  it('validates required fields', async () => {
    const wrapper = mount(Login, {
      global: {
        plugins: [createTestingPinia()],
      },
    });

    await wrapper.find('form').trigger('submit.prevent');

    await flushPromises();

    expect(axios.post).not.toHaveBeenCalled();
    expect(wrapper.find('.el-form-item__error').exists()).toBe(true);
  });
});
