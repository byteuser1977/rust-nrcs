/// <reference types="cypress" />

describe('User Login Flow', () => {
  const username = 'testuser';
  const password = 'password123';

  it('should display login page', () => {
    cy.visit('/login');
    cy.get('input[name="username"]').should('exist');
    cy.get('input[name="password"]').should('exist');
    cy.get('button[type="submit"]').should('exist');
  });

  it('should login successfully via UI', () => {
    cy.visit('/login');
    cy.get('input[name="username"]').type(username);
    cy.get('input[name="password"]').type(password);
    cy.get('button[type="submit"]').click();

    // After successful login, should redirect to dashboard
    cy.url().should('include', '/dashboard');
    // Token stored in localStorage
    cy.window().its('localStorage.nrcs_token').should('exist');
  });

  it('should show error on invalid credentials', () => {
    cy.visit('/login');
    cy.get('input[name="username"]').type('wrong');
    cy.get('input[name="password"]').type('bad');
    cy.get('button[type="submit"]').click();

    cy.get('.error-message')
      .should('be.visible')
      .and('contain.text', 'Invalid credentials');
  });
});
