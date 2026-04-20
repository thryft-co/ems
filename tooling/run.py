#!/usr/bin/env python3
"""
Enterprise Management Suite (EMS) - Unified Development Tool

A unified script to manage and orchestrate client and server workflows.
Supports build, test, lint, format, clean operations for both frontend and backend.
"""

import os
import sys
import subprocess
import shutil
from pathlib import Path
from typing import Optional, List
import click

# Project paths
PROJECT_ROOT = Path(__file__).resolve().parent.parent
CLIENT_DIR = PROJECT_ROOT / "apps" / "client"
SERVER_DIR = PROJECT_ROOT / "apps" / "server"
E2E_DIR = PROJECT_ROOT / "apps" / "e2e"
MIGRATIONS_DIR = PROJECT_ROOT / "infra" / "db" / "migrations"
MIGRATION_TEST_FILE = PROJECT_ROOT / "infra" / "db" / "tests" / "migration_tests.sql"
CONFIG_DIR = PROJECT_ROOT / "config"
CONFIG_FILE = CONFIG_DIR / ".env"
CONFIG_EXAMPLE_FILE = CONFIG_DIR / ".env.example"

# Required environment variables for development
REQUIRED_ENV_VARS = ["DATABASE_URL", "SUPABASE_URL", "SUPABASE_ANON_KEY", "JWT_SECRET"]


# Color codes for output
class Colors:
    HEADER = "\033[95m"
    OKBLUE = "\033[94m"
    OKCYAN = "\033[96m"
    OKGREEN = "\033[92m"
    WARNING = "\033[93m"
    FAIL = "\033[91m"
    ENDC = "\033[0m"
    BOLD = "\033[1m"


def print_header(text: str):
    """Print a colored header."""
    click.echo(f"{Colors.HEADER}{Colors.BOLD}=== {text} ==={Colors.ENDC}")


def print_success(text: str):
    """Print a success message."""
    click.echo(f"{Colors.OKGREEN}✓ {text}{Colors.ENDC}")


def print_warning(text: str):
    """Print a warning message."""
    click.echo(f"{Colors.WARNING}⚠ {text}{Colors.ENDC}")


def print_error(text: str):
    """Print an error message."""
    click.echo(f"{Colors.FAIL}✗ {text}{Colors.ENDC}")


def run_command(
    cmd: List[str], cwd: Optional[Path] = None, check: bool = True
) -> subprocess.CompletedProcess:
    """Run a command and return the result."""
    cmd_str = " ".join(cmd)
    cwd_str = str(cwd) if cwd else str(PROJECT_ROOT)

    click.echo(f"{Colors.OKCYAN}Running: {cmd_str} (in {cwd_str}){Colors.ENDC}")

    try:
        result = subprocess.run(cmd, cwd=cwd, check=check, capture_output=False)
        return result
    except subprocess.CalledProcessError as e:
        print_error(f"Command failed with exit code {e.returncode}: {cmd_str}")
        if check:
            sys.exit(e.returncode)
        return e


def iter_migration_directories() -> List[Path]:
    """Return ordered migration directories using Diesel-compatible naming."""
    if not MIGRATIONS_DIR.exists():
        return []

    return sorted(
        [
            path
            for path in MIGRATIONS_DIR.iterdir()
            if path.is_dir() and not path.name.startswith(".") and (path / "up.sql").exists()
        ],
        key=lambda path: path.name,
    )


def run_migrations(database_url: str, dry_run: bool = False) -> None:
    """Run SQL migrations from Diesel-style migration directories."""
    migration_dirs = iter_migration_directories()

    if not migration_dirs:
        print_warning("No migration directories found")
        return

    print_success(f"Found {len(migration_dirs)} migration(s):")
    for migration_dir in migration_dirs:
        click.echo(f"  - {migration_dir.name}")

    if dry_run:
        print_warning("Dry run mode - no migrations executed")
        return

    for migration_dir in migration_dirs:
        migration_file = migration_dir / "up.sql"
        print_header(f"Running: {migration_dir.name}")

        result = subprocess.run(
            ["psql", database_url, "-f", str(migration_file)],
            capture_output=True,
            text=True,
        )

        if result.returncode == 0:
            print_success(f"Migration {migration_dir.name} completed")
            if result.stdout:
                for line in result.stdout.strip().split("\n")[:5]:
                    if line.strip():
                        click.echo(f"  {line}")
        else:
            if "already exists" in result.stderr:
                print_warning(
                    f"Migration {migration_dir.name} - objects already exist (skipped)"
                )
            else:
                print_error(f"Migration {migration_dir.name} failed:")
                click.echo(result.stderr)


def check_prerequisites():
    """Check if required tools are installed."""
    print_header("Checking Prerequisites")

    # Check Node.js
    try:
        result = subprocess.run(["node", "--version"], capture_output=True, text=True)
        if result.returncode == 0:
            print_success(f"Node.js found: {result.stdout.strip()}")
        else:
            print_error("Node.js not found. Please install Node.js 18+")
            return False
    except FileNotFoundError:
        print_error("Node.js not found. Please install Node.js 18+")
        return False

    # Check npm
    try:
        result = subprocess.run(["npm", "--version"], capture_output=True, text=True)
        if result.returncode == 0:
            print_success(f"npm found: {result.stdout.strip()}")
        else:
            print_error("npm not found")
            return False
    except FileNotFoundError:
        print_error("npm not found")
        return False

    # Check Rust
    try:
        result = subprocess.run(["rustc", "--version"], capture_output=True, text=True)
        if result.returncode == 0:
            print_success(f"Rust found: {result.stdout.strip()}")
        else:
            print_error("Rust not found. Please install Rust 1.70+")
            return False
    except FileNotFoundError:
        print_error("Rust not found. Please install Rust 1.70+")
        return False

    # Check Cargo
    try:
        result = subprocess.run(["cargo", "--version"], capture_output=True, text=True)
        if result.returncode == 0:
            print_success(f"Cargo found: {result.stdout.strip()}")
        else:
            print_error("Cargo not found")
            return False
    except FileNotFoundError:
        print_error("Cargo not found")
        return False

    return True


def check_environment():
    """Check if required environment variables are set."""
    print_header("Checking Environment Variables")

    missing_vars = []

    # Check if config file exists
    if not CONFIG_FILE.exists():
        print_warning(f"Config file not found: {CONFIG_FILE}")
        print_warning("Please copy config/.env.example to config/.env and configure it")
        return False

    # Load environment from config/.env
    try:
        with open(CONFIG_FILE, "r") as f:
            for line in f:
                line = line.strip()
                if line and not line.startswith("#") and "=" in line:
                    key, value = line.split("=", 1)
                    if not os.getenv(key.strip()):
                        os.environ[key.strip()] = value.strip()
    except Exception as e:
        print_error(f"Failed to load config/.env: {e}")
        return False

    # Check required variables
    for var in REQUIRED_ENV_VARS:
        value = os.getenv(var)
        if not value or value in [
            "your-password",
            "your-project",
            "your-anon-key-here",
            "your-service-role-key-here",
            "your-super-secret-jwt-key-here",
        ]:
            missing_vars.append(var)
        else:
            print_success(f"{var} is configured")

    if missing_vars:
        print_error("Missing or unconfigured environment variables:")
        for var in missing_vars:
            print_error(f"  - {var}")
        print_error("Please configure these variables in config/.env")
        return False

    return True


@click.group()
@click.option(
    "--skip-checks", is_flag=True, help="Skip prerequisite and environment checks"
)
@click.pass_context
def cli(ctx, skip_checks):
    """Enterprise Management Suite (EMS) - Unified Development Tool"""
    ctx.ensure_object(dict)
    ctx.obj["skip_checks"] = skip_checks

    if not skip_checks:
        if not check_prerequisites():
            sys.exit(1)

        if not check_environment():
            print_warning("Environment checks failed. Use --skip-checks to bypass.")
            sys.exit(1)


@cli.command()
@click.option("--frontend-only", is_flag=True, help="Build only the frontend")
@click.option("--backend-only", is_flag=True, help="Build only the backend")
@click.option("--release", is_flag=True, help="Build backend in release mode")
def build(frontend_only, backend_only, release):
    """Build the client and/or server components."""
    print_header("Building EMS Components")

    if not backend_only:
        print_header("Building Frontend")
        if not CLIENT_DIR.exists():
            print_error(f"Client directory not found: {CLIENT_DIR}")
            sys.exit(1)

        # Install dependencies if node_modules doesn't exist
        if not (CLIENT_DIR / "node_modules").exists():
            print_warning("node_modules not found. Installing dependencies...")
            run_command(["npm", "ci"], cwd=CLIENT_DIR)

        # Build frontend
        run_command(["npm", "run", "build"], cwd=CLIENT_DIR)
        print_success("Frontend build completed")

    if not frontend_only:
        print_header("Building Backend")
        if not SERVER_DIR.exists():
            print_error(f"Server directory not found: {SERVER_DIR}")
            sys.exit(1)

        # Build backend
        build_cmd = ["cargo", "build"]
        if release:
            build_cmd.append("--release")

        run_command(build_cmd, cwd=SERVER_DIR)
        print_success("Backend build completed")

    print_success("Build completed successfully!")


@cli.command()
@click.option("--frontend-only", is_flag=True, help="Test only the frontend")
@click.option("--backend-only", is_flag=True, help="Test only the backend")
@click.option("--migrations-only", is_flag=True, help="Test only the database migrations")
def test(frontend_only, backend_only, migrations_only):
    """Run tests for client and/or server components."""
    print_header("Running EMS Tests")

    if not backend_only and not migrations_only:
        print_header("Running Frontend Tests")
        if CLIENT_DIR.exists():
            # Check if test script exists in package.json
            package_json = CLIENT_DIR / "package.json"
            if package_json.exists():
                import json

                with open(package_json) as f:
                    pkg = json.load(f)
                if "test" in pkg.get("scripts", {}):
                    run_command(
                        ["npm", "test", "--", "--watchAll=false"], cwd=CLIENT_DIR
                    )
                    print_success("Frontend tests completed")
                else:
                    print_warning("No test script found in frontend package.json")
            else:
                print_warning("Frontend package.json not found")
        else:
            print_warning("Frontend directory not found")

    if not frontend_only and not migrations_only:
        print_header("Running Backend Tests")
        if SERVER_DIR.exists():
            run_command(["cargo", "test"], cwd=SERVER_DIR)
            print_success("Backend tests completed")
        else:
            print_warning("Backend directory not found")

    if not frontend_only and not backend_only:
        print_header("Running Migration Tests")
        # Load config/.env if not already loaded
        if CONFIG_FILE.exists():
            try:
                with open(CONFIG_FILE, "r") as f:
                    for line in f:
                        line = line.strip()
                        if line and not line.startswith("#") and "=" in line:
                            key, value = line.split("=", 1)
                            if not os.getenv(key.strip()):
                                os.environ[key.strip()] = value.strip()
            except Exception as e:
                print_error(f"Failed to load config/.env: {e}")
                return

        database_url = os.getenv("DATABASE_URL")
        if not database_url:
            print_error("DATABASE_URL not set. Please configure config/.env")
            return

        if MIGRATION_TEST_FILE.exists():
            print_header(f"Running Migration Tests: {MIGRATION_TEST_FILE.name}")
            try:
                result = subprocess.run(
                    ["psql", database_url, "-f", str(MIGRATION_TEST_FILE)],
                    capture_output=True,
                    text=True
                )

                if result.returncode == 0:
                    print_success("Migration tests completed successfully")
                    # Show some output if there are notices
                    if result.stderr:
                        for line in result.stderr.split('\n'):
                            if 'PASS:' in line or 'NOTICE:' in line:
                                click.echo(f"  {line}")
                else:
                    print_error(f"Migration tests failed:")
                    click.echo(result.stderr)
            except FileNotFoundError:
                print_error("psql not found. Please install PostgreSQL client tools.")
            except Exception as e:
                print_error(f"Error running migration tests: {e}")
        else:
            print_warning("Migration test file not found")

    print_success("All tests completed!")


@cli.command()
@click.option("--frontend-only", is_flag=True, help="Lint only the frontend")
@click.option("--backend-only", is_flag=True, help="Lint only the backend")
@click.option(
    "--fix", is_flag=True, help="Automatically fix linting issues where possible"
)
def lint(frontend_only, backend_only, fix):
    """Run linting for client and/or server components."""
    print_header("Running EMS Linting")

    if not backend_only:
        print_header("Linting Frontend")
        if CLIENT_DIR.exists():
            lint_cmd = ["npm", "run", "lint"]
            if fix:
                lint_cmd.append("--", "--fix")

            result = run_command(lint_cmd, cwd=CLIENT_DIR, check=False)
            if result.returncode == 0:
                print_success("Frontend linting completed")
            else:
                print_warning("Frontend linting found issues")
        else:
            print_warning("Frontend directory not found")

    if not frontend_only:
        print_header("Linting Backend")
        if SERVER_DIR.exists():
            # Check code with Cargo
            run_command(["cargo", "check"], cwd=SERVER_DIR)

            # Run Clippy for additional linting
            clippy_cmd = ["cargo", "clippy", "--all-targets", "--", "-D", "warnings"]
            if fix:
                clippy_cmd = [
                    "cargo",
                    "clippy",
                    "--fix",
                    "--allow-dirty",
                    "--allow-staged",
                ]

            result = run_command(clippy_cmd, cwd=SERVER_DIR, check=False)
            if result.returncode == 0:
                print_success("Backend linting completed")
            else:
                print_warning("Backend linting found issues")
        else:
            print_warning("Backend directory not found")


@cli.command()
@click.option("--frontend-only", is_flag=True, help="Format only the frontend")
@click.option("--backend-only", is_flag=True, help="Format only the backend")
def format(frontend_only, backend_only):
    """Format code for client and/or server components."""
    print_header("Formatting EMS Code")

    if not backend_only:
        print_header("Formatting Frontend")
        if CLIENT_DIR.exists():
            # Check if format script exists
            package_json = CLIENT_DIR / "package.json"
            if package_json.exists():
                import json

                with open(package_json) as f:
                    pkg = json.load(f)
                if "format" in pkg.get("scripts", {}):
                    run_command(["npm", "run", "format"], cwd=CLIENT_DIR)
                    print_success("Frontend formatting completed")
                else:
                    print_warning("No format script found in frontend package.json")
            else:
                print_warning("Frontend package.json not found")
        else:
            print_warning("Frontend directory not found")

    if not frontend_only:
        print_header("Formatting Backend")
        if SERVER_DIR.exists():
            run_command(["cargo", "fmt"], cwd=SERVER_DIR)
            print_success("Backend formatting completed")
        else:
            print_warning("Backend directory not found")


@cli.command()
@click.option("--frontend-only", is_flag=True, help="Clean only the frontend")
@click.option("--backend-only", is_flag=True, help="Clean only the backend")
@click.option("--deep", is_flag=True, help="Deep clean including dependencies")
def clean(frontend_only, backend_only, deep):
    """Clean build artifacts for client and/or server components."""
    print_header("Cleaning EMS Build Artifacts")

    if not backend_only:
        print_header("Cleaning Frontend")
        if CLIENT_DIR.exists():
            # Remove dist directory
            dist_dir = CLIENT_DIR / "dist"
            if dist_dir.exists():
                shutil.rmtree(dist_dir)
                print_success("Removed frontend dist directory")

            # Remove node_modules if deep clean
            if deep:
                node_modules = CLIENT_DIR / "node_modules"
                if node_modules.exists():
                    shutil.rmtree(node_modules)
                    print_success("Removed node_modules directory")

                package_lock = CLIENT_DIR / "package-lock.json"
                if package_lock.exists():
                    package_lock.unlink()
                    print_success("Removed package-lock.json")
        else:
            print_warning("Frontend directory not found")

    if not frontend_only:
        print_header("Cleaning Backend")
        if SERVER_DIR.exists():
            # Clean cargo build artifacts
            run_command(["cargo", "clean"], cwd=SERVER_DIR)
            print_success("Backend cleaning completed")
        else:
            print_warning("Backend directory not found")


@cli.command()
@click.option(
    "--frontend-only", is_flag=True, help="Start only the frontend development server"
)
@click.option(
    "--backend-only", is_flag=True, help="Start only the backend development server"
)
@click.option("--production", is_flag=True, help="Start in production mode")
def dev(frontend_only, backend_only, production):
    """Start development servers for client and/or server."""
    print_header("Starting EMS Development Environment")

    if frontend_only and backend_only:
        print_error("Cannot specify both --frontend-only and --backend-only")
        sys.exit(1)

    processes = []

    try:
        if not backend_only:
            print_header("Starting Frontend Development Server")
            if CLIENT_DIR.exists():
                # Install dependencies if needed
                if not (CLIENT_DIR / "node_modules").exists():
                    print_warning("Installing frontend dependencies...")
                    run_command(["npm", "ci"], cwd=CLIENT_DIR)

                # Start frontend
                cmd = ["npm", "run", "preview" if production else "dev"]
                proc = subprocess.Popen(cmd, cwd=CLIENT_DIR)
                processes.append(("Frontend", proc))
                print_success("Frontend development server started")
            else:
                print_error("Frontend directory not found")

        if not frontend_only:
            print_header("Starting Backend Development Server")
            if SERVER_DIR.exists():
                # Start backend
                cmd = ["cargo", "run", "--bin", "server"]

                proc = subprocess.Popen(cmd, cwd=SERVER_DIR)
                processes.append(("Backend", proc))
                print_success("Backend development server started")
            else:
                print_error("Backend directory not found")

        if processes:
            print_success("Development environment started successfully!")
            print_warning("Press Ctrl+C to stop all servers")

            # Wait for all processes
            for name, proc in processes:
                proc.wait()

    except KeyboardInterrupt:
        print_warning("Shutting down development servers...")
        for name, proc in processes:
            proc.terminate()
            print_success(f"{name} server stopped")


@cli.command()
def setup():
    """Setup the development environment."""
    print_header("Setting Up EMS Development Environment")

    # Check if config/.env exists
    if not CONFIG_FILE.exists():
        if CONFIG_EXAMPLE_FILE.exists():
            print_warning("Copying config/.env.example to config/.env")
            shutil.copy(CONFIG_EXAMPLE_FILE, CONFIG_FILE)
            print_success(
                "Config file created. Please edit config/.env with your settings."
            )
        else:
            print_error("config/.env.example not found")
            sys.exit(1)

    # Install frontend dependencies
    if CLIENT_DIR.exists():
        print_header("Installing Frontend Dependencies")
        run_command(["npm", "ci"], cwd=CLIENT_DIR)
        print_success("Frontend dependencies installed")

    # Build backend
    if SERVER_DIR.exists():
        print_header("Building Backend")
        run_command(["cargo", "build"], cwd=SERVER_DIR)
        print_success("Backend built successfully")

    # Run database migrations
    print_header("Running Database Migrations")
    database_url = os.getenv("DATABASE_URL")
    if not database_url:
        print_error("DATABASE_URL not set")
        sys.exit(1)

    run_migrations(database_url)

    print_success("Database migrations completed")

    print_success("Development environment setup completed!")
    print_warning("Please configure config/.env before running the application")


@cli.command()
@click.option("--dry-run", is_flag=True, help="Show which migrations would run without executing")
def migrate(dry_run):
    """Run database migrations against Supabase."""
    print_header("Running Database Migrations")
    
    # Load config/.env if not already loaded
    if CONFIG_FILE.exists():
        try:
            with open(CONFIG_FILE, "r") as f:
                for line in f:
                    line = line.strip()
                    if line and not line.startswith("#") and "=" in line:
                        key, value = line.split("=", 1)
                        if not os.getenv(key.strip()):
                            os.environ[key.strip()] = value.strip()
        except Exception as e:
            print_error(f"Failed to load config/.env: {e}")
            sys.exit(1)
    
    if not MIGRATIONS_DIR.exists():
        print_error(f"Migrations directory not found: {MIGRATIONS_DIR}")
        sys.exit(1)
    
    # Get DATABASE_URL from environment
    database_url = os.getenv("DATABASE_URL")
    if not database_url:
        print_error("DATABASE_URL not set. Please configure config/.env")
        sys.exit(1)

    try:
        print_header("Executing Migrations")
        run_migrations(database_url, dry_run=dry_run)
        if not dry_run:
            print_success("Database migrations completed!")
    except FileNotFoundError:
        print_error("psql not found. Please install PostgreSQL client tools.")
        print_warning("Alternatively, run the SQL files manually in Supabase SQL Editor")
        sys.exit(1)
    except Exception as e:
        print_error(f"Error running migrations: {e}")
        sys.exit(1)


@cli.command()
def status():
    """Show the status of EMS components."""
    print_header("EMS Component Status")

    # Check frontend
    if CLIENT_DIR.exists():
        print_success("✓ Frontend directory found")
        if (CLIENT_DIR / "node_modules").exists():
            print_success("✓ Frontend dependencies installed")
        else:
            print_warning("⚠ Frontend dependencies not installed")

        if (CLIENT_DIR / "dist").exists():
            print_success("✓ Frontend build artifacts found")
        else:
            print_warning("⚠ Frontend not built")
    else:
        print_error("✗ Frontend directory not found")

    # Check backend
    if SERVER_DIR.exists():
        print_success("✓ Backend directory found")
        if (SERVER_DIR / "target").exists():
            print_success("✓ Backend build artifacts found")
        else:
            print_warning("⚠ Backend not built")
    else:
        print_error("✗ Backend directory not found")

    # Check config
    if CONFIG_FILE.exists():
        print_success("✓ Configuration file found")
    else:
        print_warning("⚠ Configuration file not found")


@cli.command()
@click.option("--smoke", is_flag=True, help="Run only smoke tests")
@click.option("--regression", is_flag=True, help="Run only regression tests")
@click.option("--critical", is_flag=True, help="Run only critical tests (tenant isolation)")
@click.option("--headed", is_flag=True, help="Run tests in headed browser mode")
@click.option("--ui", is_flag=True, help="Open Playwright UI mode")
@click.option("--project", type=click.Choice(["chromium", "firefox", "webkit"]), help="Run tests in a specific browser")
@click.option("--no-server", is_flag=True, help="Skip starting backend/frontend (if already running)")
def e2e(smoke, regression, critical, headed, ui, project, no_server):
    """Run end-to-end tests using Playwright."""
    import time
    import signal
    import urllib.request
    import urllib.error

    print_header("Running E2E Tests")

    if not E2E_DIR.exists():
        print_error(f"E2E testing directory not found: {E2E_DIR}")
        sys.exit(1)

    # Install E2E dependencies if needed
    if not (E2E_DIR / "node_modules").exists():
        print_warning("E2E dependencies not installed. Installing...")
        run_command(["npm", "install"], cwd=E2E_DIR)

    # Install frontend dependencies if needed
    if not no_server and not (CLIENT_DIR / "node_modules").exists():
        print_warning("Frontend dependencies not installed. Installing...")
        run_command(["npm", "install"], cwd=CLIENT_DIR)

    processes = []

    def cleanup_servers():
        """Terminate all background server processes."""
        for name, proc in processes:
            if proc.poll() is None:  # still running
                print_warning(f"Stopping {name} server (PID {proc.pid})...")
                proc.terminate()
                try:
                    proc.wait(timeout=5)
                except subprocess.TimeoutExpired:
                    proc.kill()
                print_success(f"{name} server stopped")

    def wait_for_url(url, name, max_attempts=30, delay=2):
        """Wait for a URL to become reachable."""
        for attempt in range(1, max_attempts + 1):
            try:
                req = urllib.request.urlopen(url, timeout=5)
                req.close()
                print_success(f"{name} is ready at {url} (attempt {attempt})")
                return True
            except (urllib.error.URLError, OSError):
                if attempt % 5 == 0 or attempt == 1:
                    click.echo(f"  ⏳ Waiting for {name}... (attempt {attempt}/{max_attempts})")
                time.sleep(delay)
        return False

    try:
        if not no_server:
            # ---- Start Backend ----
            print_header("Starting Backend Server")
            if not SERVER_DIR.exists():
                print_error(f"Server directory not found: {SERVER_DIR}")
                sys.exit(1)

            backend_env = os.environ.copy()
            backend_proc = subprocess.Popen(
                ["cargo", "run", "--bin", "server"],
                cwd=SERVER_DIR,
                env=backend_env,
                stdout=subprocess.DEVNULL,
                stderr=subprocess.DEVNULL,
            )
            processes.append(("Backend", backend_proc))
            print_success(f"Backend server started (PID {backend_proc.pid})")

            # Wait for backend to be healthy
            backend_url = "http://localhost:5002/health"
            if not wait_for_url(backend_url, "Backend"):
                print_error(
                    "Backend did not become healthy. Check cargo build and config/.env"
                )
                cleanup_servers()
                sys.exit(1)

            # ---- Start Frontend ----
            print_header("Starting Frontend Dev Server")
            if not CLIENT_DIR.exists():
                print_error(f"Client directory not found: {CLIENT_DIR}")
                cleanup_servers()
                sys.exit(1)

            frontend_proc = subprocess.Popen(
                ["npm", "run", "dev"],
                cwd=CLIENT_DIR,
                stdout=subprocess.DEVNULL,
                stderr=subprocess.DEVNULL,
            )
            processes.append(("Frontend", frontend_proc))
            print_success(f"Frontend dev server started (PID {frontend_proc.pid})")

            # Wait for frontend to be reachable
            frontend_url = "http://localhost:3001"
            if not wait_for_url(frontend_url, "Frontend", max_attempts=20):
                print_error("Frontend did not start. Check npm install in apps/client")
                cleanup_servers()
                sys.exit(1)

            print_success("All servers are ready!")

        # ---- Run E2E Tests ----
        print_header("Executing Playwright Tests")
        cmd = ["npx", "playwright", "test"]

        if smoke:
            cmd.extend(["--grep", "@smoke"])
        elif regression:
            cmd.extend(["--grep", "@regression"])
        elif critical:
            cmd.extend(["--grep", "@critical"])

        if headed:
            cmd.append("--headed")

        if ui:
            cmd.append("--ui")

        if project:
            cmd.extend(["--project", project])

        result = run_command(cmd, cwd=E2E_DIR, check=False)

        if result.returncode == 0:
            print_success("E2E tests passed!")
        else:
            print_error("E2E tests failed")

    except KeyboardInterrupt:
        print_warning("\nInterrupted by user")

    finally:
        # Always clean up servers
        cleanup_servers()

    if not no_server or 'result' in dir():
        try:
            if result.returncode != 0:
                sys.exit(result.returncode)
        except NameError:
            sys.exit(1)


if __name__ == "__main__":
    cli()
