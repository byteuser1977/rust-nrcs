import { describe, it, expect, vi, beforeEach } from 'vitest';
import { createTestingPinia } from '@pinia/testing';
import { setActivePinia } from 'pinia';
import { useAccountStore } from '@/stores/account';
import { useTransactionStore } from '@/stores/transaction';

describe('Account Store', () => {
  beforeEach(() => {
    setActivePinia(createTestingPinia());
  });

  it('sets token correctly', () => {
    const store = useAccountStore();
    store.setToken('test-token');

    expect(store.token).toBe('test-token');
    // Should also set in localStorage
    expect(localStorage.getItem('nrcs_token')).toBe('test-token');
  });

  it('clears token on logout', () => {
    const store = useAccountStore();
    store.setToken('test-token');
    store.logout();

    expect(store.token).toBeNull();
    expect(store.user).toBeNull();
    expect(localStorage.getItem('nrcs_token')).toBeNull();
  });

  it('sets user info correctly', () => {
    const store = useAccountStore();
    const user = { id: 1, username: 'alice', balance: 1000000 };

    store.setUser(user);

    expect(store.user).toEqual(user);
  });

  it('updates balance correctly', () => {
    const store = useAccountStore();
    store.setUser({ id: 1, username: 'test', balance: 1000 });

    store.updateBalance(500);

    expect(store.user?.balance).toBe(1500);
  });
});

describe('Transaction Store', () => {
  beforeEach(() => {
    setActivePinia(createTestingPinia());
  });

  it('adds transaction to recent list', () => {
    const store = useTransactionStore();

    const tx = {
      id: 'abc123',
      type: 'PAYMENT',
      amount: 100,
      status: 'PENDING',
      timestamp: Date.now(),
    };

    store.addTransaction(tx);

    expect(store.recentTransactions).toHaveLength(1);
    expect(store.recentTransactions[0]).toEqual(tx);
  });

  it('removes old transactions when exceeding limit', () => {
    const store = useTransactionStore();

    // Add 11 transactions (default limit is 10)
    for (let i = 0; i < 11; i++) {
      store.addTransaction({
        id: `tx${i}`,
        type: 'PAYMENT',
        amount: 100,
        status: 'PENDING',
        timestamp: Date.now(),
      });
    }

    expect(store.recentTransactions).toHaveLength(10);
    // Oldest transaction should be removed
    expect(store.recentTransactions.find(tx => tx.id === 'tx0')).toBeUndefined();
    expect(store.recentTransactions.find(tx => tx.id === 'tx10')).toBeDefined();
  });

  it('updates transaction status', () => {
    const store = useTransactionStore();

    store.addTransaction({
      id: 'abc123',
      type: 'PAYMENT',
      amount: 100,
      status: 'PENDING',
      timestamp: Date.now(),
    });

    store.updateTransactionStatus('abc123', 'CONFIRMED');

    const updated = store.recentTransactions.find(tx => tx.id === 'abc123');
    expect(updated?.status).toBe('CONFIRMED');
  });

  it('sets loading state correctly', () => {
    const store = useTransactionStore();

    store.setLoading(true);
    expect(store.loading).toBe(true);

    store.setLoading(false);
    expect(store.loading).toBe(false);
  });

  it('handles error state', () => {
    const store = useTransactionStore();

    store.setError('Network error');
    expect(store.error).toBe('Network error');

    store.clearError();
    expect(store.error).toBeNull();
  });
});
