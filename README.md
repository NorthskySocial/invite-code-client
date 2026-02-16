# Invite Code Client

Frontend UI Client to interact with the Invite Code Manager. Built with React and TypeScript.

## Getting Started

### Prerequisites

- Node.js (v18 or later recommended)
- npm

### Installation

```bash
npm install
```

### Development

```bash
npm run dev
```

### Production Build (Web)

```bash
npm run build
```

### Desktop Executable

To generate a local executable for your current platform:

```bash
npm run electron:build
```

The executable will be located in the `release` directory.

To run the application in desktop mode for development:

```bash
npm run electron:dev
```

### Local HTTPS Development

This project supports local HTTPS development using [Caddy](https://caddyserver.com/).

1. **Install Caddy**: Ensure you have Caddy installed on your system.
2. **Update Hosts File**: Add the following line to your `/etc/hosts` (or
   `C:\Windows\System32\drivers\etc\hosts` on Windows):
   ```
   127.0.0.1 frontend.myapp.local
   ```
3. **Run Development Server**:
   ```bash
   npm run dev
   ```
4. **Run Caddy**: In a new terminal window, run:
   ```bash
   caddy run
   ```
5. **Access the site**: Open [https://frontend.myapp.local](https://frontend.myapp.local) in your
   browser. Caddy will automatically generate and trust a local certificate (you may be prompted for
   your password).

## License

MIT
