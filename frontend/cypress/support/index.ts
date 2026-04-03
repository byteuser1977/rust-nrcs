// Cypress support file
// You can add global before/after hooks, custom commands, etc.

// Example: custom command to login via API
Cypress.Commands.add('login', (username: string, password: string) => {
  cy.request({
    method: 'POST',
    url: `${Cypress.env('apiUrl')}/auth/login`,
    body: { username, password },
  }).then((resp) => {
    expect(resp.status).to.eq(200);
    const token = resp.body.token;
    window.localStorage.setItem('nrcs_token', token);
  });
});

// Preserve token between tests
beforeEach(() => {
  const token = window.localStorage.getItem('nrcs_token');
  if (token) {
    // Ensure auth header is attached automatically via interceptor
    Cypress.on('request', (req) => {
      req.headers['Authorization'] = `Bearer ${token}`;
    });
  }
});
