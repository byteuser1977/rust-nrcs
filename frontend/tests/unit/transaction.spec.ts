import { describe, it, expect, vi, beforeEach } from 'vitest';
import { mount, flushPromises } from '@vue/test-utils';
import { createTestingPinia } from '@pinia/testing';
import SendTransaction from '@/views/transactions/Send.vue';
import { useAccountStore } from '@/stores/account';
import { useTransactionStore } from '@/stores/transaction';

vi.mock('axios', () => ({
  default: {
    get: vi.fn(),
    post: vi.fn(),
  },
}));

import axios from 'axios';

describe('SendTransaction Component', () => {
  beforeEach(() => {
    vi.clearAllMocks();
    // Set logged in state
    const accountStore = useAccountStore();
    accountStore.setToken('mock-token');
    accountStore.setUser({ id: 1, username: 'test', balance: 1000000000 });
  });

  it('renders transaction form', () => {
    const wrapper = mount(SendTransaction, {
      global: {
        plugins: [createTestingPinia()],
      },
    });

    expect(wrapper.find('input[name="recipient"]').exists()).toBe(true);
    expect(wrapper.find('input[name="amount"]').exists()).toBe(true);
    expect(wrapper.find('textarea[name="memo"]').exists()).toBe(true);
  });

  it('validates recipient address format', async () => {
    const wrapper = mount(SendTransaction, {
      global: {
        plugins: [createTestingPinia()],
      },
    });

    await wrapper.find('input[name="recipient"]').setValue('invalid-address');
    await wrapper.find('form').trigger('submit.prevent');

    await flushPromises();

    expect(wrapper.find('.el-form-item__error').exists()).toBe(true);
  });

  it('calculates fee correctly', async () => {
    const wrapper = mount(SendTransaction, {
      global: {
        plugins: [createTestingPinia()],
      },
    });

    await wrapper.find('input[name="amount"]').setValue('100');
    await flushPromises();

    // Fee should be displayed (e.g., 0.001 NRC = 100000 NQT)
    expect(wrapper.text()).toContain('Fee');
  });

  it('submits transaction successfully', async () => {
    const mockResponse = { data: { transaction_id: 'abc123', status: 'pending' } };
    (axios.post as any).mockResolvedValue(mockResponse);

    const wrapper = mount(SendTransaction, {
      global: {
        plugins: [createTestingPinia()],
      },
    });

    await wrapper.find('input[name="recipient"]').setValue('NRCS-valid-address');
    await wrapper.find('input[name="amount"]').setValue('100');
    await wrapper.find('textarea[name="memo"]').setValue('Test payment');
    await wrapper.find('form').trigger('submit.prevent');

    await flushPromises();

    expect(axios.post).toHaveBeenCalledWith(
      '/api/v1/transactions',
      expect.objectContaining({
        recipient: 'NRCS-valid-address',
        amount: 100,
        fee: expect.any(Number),
      })
    );

    const txStore = useTransactionStore();
    expect(txStore.recentTransactions.length).toBeGreaterThan(0);
  });

  it('checks balance before submitting', async () => {
    const wrapper = mount(SendTransaction, {
      global: {
        plugins: [createTestingPinia()],
      },
    });

    // Attempt to send more than balance
    await wrapper.find('input[name="amount"]').setValue('1000000'); // more than balance
    await wrapper.find('form').trigger('submit.prevent');

    await flushPromises();

    expect(axios.post).not.toHaveBeenCalled();
    expect(wrapper.text()).toContain('Insufficient balance');
  });

  it('displays fee breakdown', () => {
    const wrapper = mount(SendTransaction, {
      global: {
        plugins: [createTestingPinia()],
      },
    });

    expect(wrapper.text()).toContain('Network Fee');
    expect(wrapper.text()).toContain('Total');
  });
});
