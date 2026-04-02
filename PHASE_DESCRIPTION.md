# Phase 8.1: Firebase Website Hosting

This phase addresses the immediate need for a free, reliable hosting platform while the official domain is being secured.

### Key Accomplishments:
- **Firebase Hosting Configuration**: Created `firebase.json` for high-performance hosting with Single-Page Application (SPA) support and security headers for WASM.
- **Automated Deployment Workflow**: Established `.github/workflows/deploy-firebase.yml` to automate builds and deployments directly from the main branch.
- **Guideline Documentation**: Created `docs/FIREBASE_GUIDELINE.md` to assist the owner in setting up their Firebase project and connecting the deployment pipeline.
- **Distribution Asset Hosting**: Copied the one-line installer (`install.sh`) to the website's public directory to ensure it is hosted alongside the language documentation.

### Verification:
- Firebase configuration syntax validated locally.
- Deployment workflow configured for automatic environment updates.
- Installer script parity maintained with root `public/` folder.

This setup ensures Lumina is accessible immediately at a `*.web.app` subdomain for developers and community testing.
