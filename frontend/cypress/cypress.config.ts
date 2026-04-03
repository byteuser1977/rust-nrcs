import { defineConfig } from 'cypress';

export default defineConfig({
  e2e: {
    baseUrl: 'http://localhost', // dev server URL
    specPattern: 'cypress/e2e/**/*.cy.{js,jsx,ts,tsx}',
    supportFile: 'cypress/support/index.ts',
    viewportWidth: 1280,
    viewportHeight: 720,
    video: false,
    screenshotsFolder: 'cypress/screenshots',
    videosFolder: 'cypress/videos',
    defaultCommandTimeout: 8000,
    pageLoadTimeout: 60000,
    env: {
      apiUrl: 'http://localhost:17976/api/v1',
    },
  },
});
