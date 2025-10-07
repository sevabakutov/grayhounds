# Dependencies install
init:
	npm install
	cargo fetch --manifest-path src-tauri/Cargo.toml

# Run dev-server (frontend + tauri)
dev:
	npm run dev

# Full project build (tsc + vite)
build:
	npm run build
	cargo tauri build

# Just frontend build
build-frontend:
	npm run build

# Just backend build
build-backend:
	cargo tauri build

# Preview build (vite preview)
preview:
	npm run preview

# Call Tauri CLI with arguments
tauri *args:
	npm run tauri -- {{args}}