# Publishing to VS Code Marketplace

You've already generated the `.vsix` package in `extensions/lumina-vscode/lumina-lang-1.7.0.vsix`. Here are the steps to make it live for the world.

## 1. Prerequisites
- A **Microsoft Account**.
- An **Azure DevOps organization** (to generate a Personal Access Token).
- A **Publisher ID** (set as `luminalang` in our `package.json`).

## 2. Generate a Personal Access Token (PAT)
1. Go to [dev.azure.com](https://dev.azure.com/) and sign in.
2. Click on the **User Settings** icon (top right) -> **Personal Access Tokens**.
3. Create a **New Token**.
4. Set the name to `lumina-vsce`.
5. Under **Organization**, select `All accessible organizations`.
6. Under **Scopes**, select `Custom defined`.
7. Scroll down to **Marketplace** and check `Publish`.
8. **Copy your token immediately**. You won't see it again!

## 3. Create a Publisher
If you haven't already:
1. Go to the [VS Code Marketplace Management Console](https://marketplace.visualstudio.com/manage).
2. Create a new publisher with the ID: `luminalang`.

## 4. Method A: Command Line (Fastest)
Run the following commands in `extensions/lumina-vscode`:

```bash
# Login to vsce with your PAT
npx @vscode/vsce login luminalang

# Publish the extension
npx @vscode/vsce publish
```

## 5. Method B: Visual Upload (Easiest for First Time)
1. Go to the [Management Console](https://marketplace.visualstudio.com/manage).
2. Click on your `luminalang` publisher.
3. Click **New Extension** -> **Visual Studio Code**.
4. Drag and drop your `lumina-lang-1.7.0.vsix` file.
5. Review the details and click **Upload**.

## 6. Verification
Once uploaded, the extension will undergo a brief automated verification. It usually goes live within a few minutes. You can then search for "Lumina" in the VS Code Extensions view!