describe('Dashboard', () => {
  beforeEach(() => {
    // Mock login state
    localStorage.setItem('user', JSON.stringify({
      address: '0x1234567890abcdef',
      balance: 1000
    }))
    cy.visit('/dashboard')
  })

  it('should display user balance', () => {
    cy.get('[data-testid="balance"]').should('contain', '1000')
  })

  it('should display latest blocks', () => {
    // Mock API response
    cy.intercept('GET', '/api/v1/blocks/latest', {
      statusCode: 200,
      body: {
        code: 0,
        data: {
          height: 100,
          hash: '0xabcdef123456',
          timestamp: Date.now() / 1000,
          transactions: []
        }
      }
    })

    cy.get('[data-testid="latest-block"]').should('exist')
    cy.get('[data-testid="block-height"]').should('contain', '100')
  })

  it('should display node status', () => {
    cy.intercept('GET', '/api/v1/node/info', {
      statusCode: 200,
      body: {
        code: 0,
        data: {
          version: '1.0.0',
          peers: 10,
          height: 100
        }
      }
    })

    cy.get('[data-testid="node-status"]').should('exist')
    cy.get('[data-testid="peer-count"]').should('contain', '10')
  })

  it('should allow navigation to transaction page', () => {
    cy.get('[data-testid="nav-transaction"]').click()
    cy.url().should('include', '/transaction')
  })
})