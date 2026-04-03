/// <reference types="cypress" />

describe('Transaction Send Flow', () => {
  const sender = 'testuser';
  const password = 'password123';
  const recipient = 'NRCS-recipient-test';

  before(() => {
    // Ensure we are logged in via API before tests
    cy.login(sender, password);
  });

  it('should navigate to Send Transaction page', () => {
    cy.visit('/transactions/send');
    cy.url().should('include', '/transactions/send');
    cy.get('input[name="recipient"]').should('exist');
    cy.get('input[name="amount"]').should('exist');
  });

  it('should validate insufficient balance', () => {
    cy.visit('/transactions/send');
    cy.get('input[name="recipient"]').type(recipient);
    cy.get('input[name="amount"]').type('1000000000000'); // huge amount
    cy.get('button[type="submit"]').click();
    cy.get('.error-message')
      .should('be.visible')
      .and('contain.text', 'Insufficient balance');
  });

  it('should submit a valid transaction', () => {
    cy.visit('/transactions/send');
    cy.get('input[name="recipient"]').type(recipient);
    cy.get('input[name="amount"]').type('10'); // 10 NQT, small amount
    cy.get('textarea[name="memo"]').type('Test payment');
    // Ensure fee is displayed and >0
    cy.get('.fee-display').should('contain.text', 'Fee');
    cy.get('button[type="submit"]').click();

    // Intercept the API call to ensure it succeeded
    cy.intercept('POST', '/api/v1/transactions').as('submitTx');
    cy.wait('@submitTx').its('response.statusCode').should('eq', 201);

    // After success, should redirect to transaction list
    cy.url().should('include', '/transactions');
    cy.contains('Transaction submitted').should('be.visible');
  });
});
