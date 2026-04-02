# Guideline: Hosting Lumina on Firebase (Free)

Since we are using Firebase Hosting's free tier, we will use the `*.web.app` subdomain (e.g., `lumina-lang.web.app`).

## 1. Local Setup
Since you have already run `firebase login`, the next steps are:

```bash
# Initialize hosting in the root directory
firebase init hosting
```

**Selection Details:**
- **Project Selection**: Select the project you created in the console (e.g., `lumina-lang`).
- **Public Directory**: Set this to `website/dist` (Vite's default output).
- **Configure as a single-page app?**: Yes.
- **Set up automatic builds and deploys with GitHub?**: No (we will use a custom GitHub Action for more control).

## 2. GitHub Configuration
After initializing, you need to allow GitHub to deploy on your behalf.

1. **Service Account**: Go to the Firebase Console > Project Settings > Service Accounts.
2. **Generate Key**: Generate a new private key and download the JSON.
3. **GitHub Secret**: Go to your GitHub repository > Settings > Secrets > Actions.
4. **Create Secret**: Add a new secret named `FIREBASE_SERVICE_ACCOUNT_LUMINA_LANG` and paste the entire JSON key content.

## 3. Automation
I have created the `.github/workflows/deploy-firebase.yml` file which will automatically:
- Build the website using `npm run build` in the `website/` directory.
- Deploy to the `live` channel on every push to `main`.
- Deploy to a `preview` channel on every Pull Request.

## 4. Temporary URL
Your website will be live at:
`https://<your-project-id>.web.app`

You can use this URL for all documentation and installer links until `lumina-lang.dev` is registered.
