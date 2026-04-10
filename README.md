# Enterprise Management Suite

[![CI](https://github.com/shishir-dey/ems/actions/workflows/ci.yml/badge.svg)](https://github.com/shishir-dey/ems/actions/workflows/ci.yml) [![CD](https://github.com/shishir-dey/ems/actions/workflows/cd.yml/badge.svg)](https://github.com/shishir-dey/ems/actions/workflows/cd.yml)

A modern, multi-tenant Enterprise Management Suite for hardware manufacturing

### Tech Stack

- **Backend**: Rust + Axum + Diesel + PostgreSQL
- **Frontend**: React + TypeScript + Tailwind CSS + Vite
- **Database**: Supabase PostgreSQL with Row Level Security
- **Authentication**: Supabase Auth with JWT tokens

### Features

- **Multi-tenant Architecture**: Complete tenant isolation
- **Person Management**: Internal, Customer, Vendor, and Distributor types
- **Job Management**: Manufacturing, QA, and Service jobs
- **RESTful API**: Type-safe with comprehensive validation
- **Modern UI**: Responsive design with accessible components

### Project Structure

```
├── packages/
│   ├── ems-client/      # React frontend (TypeScript + Vite)
│   ├── ems-server/      # Rust backend (Axum + Diesel)
│   ├── ems-db/          # Database migrations (PostgreSQL)
│   ├── ems-docker/      # Docker configuration
│   ├── ems-docs/        # API documentation
│   ├── ems-e2e-testing/ # End-to-end tests
│   ├── ems-nginx/       # Nginx configuration
│   └── ems-website/     # Static website
├── .github/             # GitHub Actions CI/CD
├── config.env.example   # Environment configuration template
├── requirements.txt     # Python dependencies
├── run.py               # Unified development script
├── .dockerignore        # Docker ignore rules
├── .gitignore           # Git ignore rules
└── README.md            # This file
```

### Quick Start

#### Prerequisites
- Node.js 18+ and npm
- Rust 1.70+ and Cargo
- Python 3.7+ and pip
- Supabase account

#### Setup

```bash
git clone <repository-url>
cd ems
pip install -r requirements.txt
python run.py setup
```

Configure `config.env` with your Supabase credentials, then start development:

```bash
python run.py dev
```
   
### Development Commands

| Command | Description | Options |
|---------|-------------|---------|
| `python run.py setup` | Setup development environment | |
| `python run.py dev` | Start development servers | `--frontend-only`, `--backend-only` |
| `python run.py build` | Build frontend and backend | `--frontend-only`, `--backend-only`, `--release` |
| `python run.py test` | Run all tests | `--frontend-only`, `--backend-only` |
| `python run.py lint` | Lint code | `--frontend-only`, `--backend-only`, `--fix` |
| `python run.py format` | Format code | `--frontend-only`, `--backend-only` |
| `python run.py clean` | Clean build artifacts | `--frontend-only`, `--backend-only`, `--deep` |
| `python run.py e2e` | Run E2E tests (Playwright) | `--smoke`, `--regression`, `--critical`, `--headed`, `--ui`, `--project`, `--no-server` |
| `python run.py status` | Show component status | |

### E2E Testing

The E2E test suite lives in `packages/ems-e2e-testing/` and uses [Playwright](https://playwright.dev/).

```bash
# Run all E2E tests
python run.py e2e

# Run only smoke tests (auth flows)
python run.py e2e --smoke

# Run regression tests (CRUD operations)
python run.py e2e --regression

# Run critical tests (tenant isolation)
python run.py e2e --critical

# Run in headed mode (visible browser)
python run.py e2e --headed

# Open Playwright UI
python run.py e2e --ui

# Run in a specific browser
python run.py e2e --project chromium
```
