/// <reference types="cypress" />

describe('Block Explorer', () => {
  const username = 'testuser';
  const password = 'password123';

  before(() => {
    cy.login(username, password);
  });

  it('should display latest block on dashboard', () => {
    cy.visit('/dashboard');
    cy.get('.latest-block')
      .should('exist')
      .and('contain.text', 'Block Height');
  });

  it('should navigate to block explorer page', () => {
    cy.visit('/explorer');
    cy.url().should('include', '/explorer');
    cy.get('.block-list').should('exist');
  });

  it('should view details of a specific block', () => {
    // Assume first block in the list is clickable and has data-block-id attribute
    cy.visit('/explorer');
    cy.get('.block-list .block-item')
      .first()
      .invoke('attr', 'data-block-id')
      .then((blockId) => {
        cy.wrap(blockId).should('not.be.empty');
        cy.get(`.block-item[data-block-id="${blockId}"]`).click();
        // Verify navigation to block detail page
        cy.url().should('include', `/explorer/${blockId}`);
        // Verify block details
        cy.get('.block-detail').should('contain.text', `Block #${blockId}`);
        cy.get('.transactions').should('exist');
      });
  });
});
