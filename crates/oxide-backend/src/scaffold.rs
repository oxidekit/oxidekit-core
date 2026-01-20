//! Project scaffolding for backend stacks
//!
//! Generates complete backend project structures for supported stacks:
//! - FastAPI (Python)
//! - Axum (Rust) - future
//! - Actix (Rust) - future

use crate::{BackendError, Result};
use camino::Utf8PathBuf;
use serde::{Deserialize, Serialize};
use std::path::Path;

/// Supported backend stacks
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum BackendStack {
    /// FastAPI (Python) - default and recommended
    #[default]
    FastApi,
    /// Axum (Rust) - future support
    Axum,
    /// Actix-web (Rust) - future support
    Actix,
}

impl BackendStack {
    /// Get the display name for the stack
    pub fn display_name(&self) -> &'static str {
        match self {
            BackendStack::FastApi => "FastAPI",
            BackendStack::Axum => "Axum",
            BackendStack::Actix => "Actix-web",
        }
    }

    /// Check if the stack is currently supported
    pub fn is_supported(&self) -> bool {
        matches!(self, BackendStack::FastApi)
    }
}

/// Database configuration for scaffolding
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct DatabaseConfig {
    /// Database type
    pub db_type: DatabaseType,
    /// Include migrations setup
    pub migrations: bool,
}

/// Supported database types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum DatabaseType {
    /// No database
    #[default]
    None,
    /// PostgreSQL
    Postgres,
    /// SQLite
    Sqlite,
    /// MySQL
    Mysql,
}

/// Project configuration for scaffolding
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectConfig {
    /// Project name (used for directory and package name)
    pub name: String,
    /// Project description
    pub description: String,
    /// Backend stack to use
    pub stack: BackendStack,
    /// Database configuration
    pub database: DatabaseConfig,
    /// Include authentication module
    pub include_auth: bool,
    /// API version prefix
    pub api_version: String,
    /// Port to run on
    pub port: u16,
}

impl ProjectConfig {
    /// Create a new project configuration with sensible defaults
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            description: String::new(),
            stack: BackendStack::default(),
            database: DatabaseConfig::default(),
            include_auth: true,
            api_version: "v1".to_string(),
            port: 8000,
        }
    }

    /// Set the project description
    pub fn with_description(mut self, description: impl Into<String>) -> Self {
        self.description = description.into();
        self
    }

    /// Set the backend stack
    pub fn with_stack(mut self, stack: BackendStack) -> Self {
        self.stack = stack;
        self
    }

    /// Configure database
    pub fn with_database(mut self, db_type: DatabaseType, migrations: bool) -> Self {
        self.database = DatabaseConfig { db_type, migrations };
        self
    }

    /// Enable or disable auth module
    pub fn with_auth(mut self, include_auth: bool) -> Self {
        self.include_auth = include_auth;
        self
    }
}

/// Project scaffolder that generates backend projects
#[derive(Debug)]
pub struct ProjectScaffold {
    config: ProjectConfig,
}

impl ProjectScaffold {
    /// Create a new project scaffolder
    pub fn new(config: ProjectConfig) -> Self {
        Self { config }
    }

    /// Generate the project in the specified directory
    pub fn generate(&self, output_dir: impl AsRef<Path>) -> Result<Utf8PathBuf> {
        let output = output_dir.as_ref().join(&self.config.name);
        let output_utf8 =
            Utf8PathBuf::from_path_buf(output.clone()).map_err(|p| BackendError::Io(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                format!("Invalid UTF-8 path: {:?}", p),
            )))?;

        if !self.config.stack.is_supported() {
            return Err(BackendError::Config(format!(
                "Stack {} is not yet supported",
                self.config.stack.display_name()
            )));
        }

        // Create base directory structure
        std::fs::create_dir_all(&output)?;

        match self.config.stack {
            BackendStack::FastApi => self.generate_fastapi(&output_utf8)?,
            _ => {
                return Err(BackendError::Config(format!(
                    "Stack {} is not yet implemented",
                    self.config.stack.display_name()
                )))
            }
        }

        tracing::info!("Generated {} project at {}", self.config.stack.display_name(), output_utf8);
        Ok(output_utf8)
    }

    /// Generate FastAPI project structure
    fn generate_fastapi(&self, output: &Utf8PathBuf) -> Result<()> {
        // Create directory structure
        let dirs = [
            "app",
            "app/api",
            "app/api/v1",
            "app/core",
            "app/models",
            "app/schemas",
            "app/services",
            "tests",
            "tests/api",
        ];

        for dir in dirs {
            std::fs::create_dir_all(output.join(dir))?;
        }

        // Generate pyproject.toml
        self.write_file(output, "pyproject.toml", &self.fastapi_pyproject())?;

        // Generate requirements.txt
        self.write_file(output, "requirements.txt", &self.fastapi_requirements())?;

        // Generate main.py
        self.write_file(output, "app/main.py", &self.fastapi_main())?;

        // Generate config
        self.write_file(output, "app/core/config.py", &self.fastapi_config())?;

        // Generate API router
        self.write_file(output, "app/api/v1/__init__.py", "")?;
        self.write_file(output, "app/api/v1/router.py", &self.fastapi_router())?;

        // Generate health endpoint
        self.write_file(output, "app/api/v1/health.py", &self.fastapi_health())?;

        // Generate auth endpoints if enabled
        if self.config.include_auth {
            self.write_file(output, "app/api/v1/auth.py", &self.fastapi_auth())?;
            self.write_file(output, "app/schemas/auth.py", &self.fastapi_auth_schemas())?;
            self.write_file(output, "app/services/auth.py", &self.fastapi_auth_service())?;
        }

        // Generate empty __init__.py files
        for path in ["app", "app/api", "app/core", "app/models", "app/schemas", "app/services", "tests", "tests/api"] {
            self.write_file(output, &format!("{}/__init__.py", path), "")?;
        }

        // Generate Dockerfile
        self.write_file(output, "Dockerfile", &self.fastapi_dockerfile())?;

        // Generate docker-compose.yml
        self.write_file(output, "docker-compose.yml", &self.fastapi_docker_compose())?;

        // Generate .env.example
        self.write_file(output, ".env.example", &self.fastapi_env_example())?;

        // Generate .gitignore
        self.write_file(output, ".gitignore", &self.fastapi_gitignore())?;

        // Generate OpenAPI stub
        self.write_file(output, "openapi.json", &self.openapi_stub())?;

        // Generate tests
        self.write_file(output, "tests/conftest.py", &self.fastapi_conftest())?;
        self.write_file(output, "tests/api/test_health.py", &self.fastapi_test_health())?;

        if self.config.include_auth {
            self.write_file(output, "tests/api/test_auth.py", &self.fastapi_test_auth())?;
        }

        Ok(())
    }

    fn write_file(&self, base: &Utf8PathBuf, path: &str, content: &str) -> Result<()> {
        let full_path = base.join(path);
        std::fs::write(&full_path, content)?;
        Ok(())
    }

    fn fastapi_pyproject(&self) -> String {
        format!(
            r#"[project]
name = "{name}"
version = "0.1.0"
description = "{description}"
requires-python = ">=3.11"
dependencies = [
    "fastapi>=0.109.0",
    "uvicorn[standard]>=0.27.0",
    "pydantic>=2.5.0",
    "pydantic-settings>=2.1.0",
    "python-jose[cryptography]>=3.3.0",
    "passlib[bcrypt]>=1.7.4",
    "python-multipart>=0.0.6",
]

[project.optional-dependencies]
dev = [
    "pytest>=8.0.0",
    "pytest-asyncio>=0.23.0",
    "httpx>=0.26.0",
    "ruff>=0.1.0",
    "mypy>=1.8.0",
]

[tool.ruff]
line-length = 100
target-version = "py311"

[tool.ruff.lint]
select = ["E", "F", "I", "N", "W", "UP"]

[tool.mypy]
python_version = "3.11"
strict = true
"#,
            name = self.config.name,
            description = self.config.description
        )
    }

    fn fastapi_requirements(&self) -> String {
        r#"fastapi>=0.109.0
uvicorn[standard]>=0.27.0
pydantic>=2.5.0
pydantic-settings>=2.1.0
python-jose[cryptography]>=3.3.0
passlib[bcrypt]>=1.7.4
python-multipart>=0.0.6
"#.to_string()
    }

    fn fastapi_main(&self) -> String {
        let auth_import = if self.config.include_auth {
            "\nfrom app.api.v1 import auth"
        } else {
            ""
        };
        let auth_router = if self.config.include_auth {
            "\n    app.include_router(auth.router, prefix=\"/auth\", tags=[\"auth\"])"
        } else {
            ""
        };

        format!(
            r#"""
{name} - FastAPI Backend

Generated by OxideKit Backend Builder.
This is a contract-first API - the OpenAPI spec is the source of truth.
"""
from contextlib import asynccontextmanager
from typing import AsyncGenerator

from fastapi import FastAPI
from fastapi.middleware.cors import CORSMiddleware

from app.api.v1 import health{auth_import}
from app.core.config import settings


@asynccontextmanager
async def lifespan(app: FastAPI) -> AsyncGenerator[None, None]:
    """Application lifespan handler."""
    # Startup
    yield
    # Shutdown


app = FastAPI(
    title=settings.PROJECT_NAME,
    description=settings.PROJECT_DESCRIPTION,
    version=settings.VERSION,
    openapi_url=f"/{{settings.API_V1_PREFIX}}/openapi.json",
    docs_url=f"/{{settings.API_V1_PREFIX}}/docs",
    redoc_url=f"/{{settings.API_V1_PREFIX}}/redoc",
    lifespan=lifespan,
)

# CORS middleware
app.add_middleware(
    CORSMiddleware,
    allow_origins=settings.CORS_ORIGINS,
    allow_credentials=True,
    allow_methods=["*"],
    allow_headers=["*"],
)


# Include routers
app.include_router(health.router, prefix="/health", tags=["health"]){auth_router}


@app.get("/")
async def root() -> dict[str, str]:
    """Root endpoint."""
    return {{"message": "Welcome to {name}", "docs": f"/{{settings.API_V1_PREFIX}}/docs"}}
"#,
            name = self.config.name,
            auth_import = auth_import,
            auth_router = auth_router
        )
    }

    fn fastapi_config(&self) -> String {
        format!(
            r#""""
Application configuration using pydantic-settings.
"""
from pydantic_settings import BaseSettings


class Settings(BaseSettings):
    """Application settings loaded from environment variables."""

    # Project info
    PROJECT_NAME: str = "{name}"
    PROJECT_DESCRIPTION: str = "{description}"
    VERSION: str = "0.1.0"
    API_V1_PREFIX: str = "/{api_version}"

    # Server
    HOST: str = "0.0.0.0"
    PORT: int = {port}
    DEBUG: bool = False

    # CORS
    CORS_ORIGINS: list[str] = ["http://localhost:3000", "http://localhost:5173"]

    # Auth
    SECRET_KEY: str = "your-secret-key-change-in-production"
    ACCESS_TOKEN_EXPIRE_MINUTES: int = 15
    REFRESH_TOKEN_EXPIRE_DAYS: int = 7
    ALGORITHM: str = "HS256"

    class Config:
        env_file = ".env"
        case_sensitive = True


settings = Settings()
"#,
            name = self.config.name,
            description = self.config.description,
            api_version = self.config.api_version,
            port = self.config.port
        )
    }

    fn fastapi_router(&self) -> String {
        r#""""
API v1 router configuration.
"""
from fastapi import APIRouter

router = APIRouter()
"#.to_string()
    }

    fn fastapi_health(&self) -> String {
        r#""""
Health check endpoints.
"""
from fastapi import APIRouter
from pydantic import BaseModel


router = APIRouter()


class HealthResponse(BaseModel):
    """Health check response - uses camelCase per OxideKit naming convention."""

    status: str
    version: str


@router.get("", response_model=HealthResponse)
async def health_check() -> HealthResponse:
    """Check service health."""
    from app.core.config import settings

    return HealthResponse(status="healthy", version=settings.VERSION)


@router.get("/ready")
async def readiness_check() -> dict[str, str]:
    """Check if service is ready to accept traffic."""
    return {"status": "ready"}


@router.get("/live")
async def liveness_check() -> dict[str, str]:
    """Check if service is alive."""
    return {"status": "alive"}
"#.to_string()
    }

    fn fastapi_auth(&self) -> String {
        r#""""
Authentication endpoints following OxideKit auth contract.

Standard endpoints:
- POST /auth/login - Login and get tokens
- POST /auth/refresh - Refresh access token
- POST /auth/logout - Logout and invalidate tokens
- POST /auth/revoke - Revoke all sessions
- GET /auth/me - Get current user info
"""
from fastapi import APIRouter, Depends, HTTPException, status
from fastapi.security import HTTPBearer

from app.schemas.auth import (
    LoginRequest,
    LoginResponse,
    RefreshRequest,
    RefreshResponse,
    UserResponse,
)
from app.services.auth import AuthService

router = APIRouter()
security = HTTPBearer()


def get_auth_service() -> AuthService:
    """Dependency to get auth service."""
    return AuthService()


@router.post("/login", response_model=LoginResponse)
async def login(
    request: LoginRequest,
    auth_service: AuthService = Depends(get_auth_service),
) -> LoginResponse:
    """
    Login with credentials and receive access/refresh tokens.

    Access tokens are short-lived (15 min default).
    Refresh tokens are longer-lived (7 days default) and rotate on use.
    """
    result = await auth_service.login(request.email, request.password)
    if result is None:
        raise HTTPException(
            status_code=status.HTTP_401_UNAUTHORIZED,
            detail="Invalid credentials",
        )
    return result


@router.post("/refresh", response_model=RefreshResponse)
async def refresh_token(
    request: RefreshRequest,
    auth_service: AuthService = Depends(get_auth_service),
) -> RefreshResponse:
    """
    Refresh access token using refresh token.

    The refresh token rotates on each use. Reuse of old refresh tokens
    will invalidate the entire session (security measure).
    """
    result = await auth_service.refresh(request.refreshToken)
    if result is None:
        raise HTTPException(
            status_code=status.HTTP_401_UNAUTHORIZED,
            detail="Invalid or expired refresh token",
        )
    return result


@router.post("/logout")
async def logout(
    auth_service: AuthService = Depends(get_auth_service),
    token: str = Depends(security),
) -> dict[str, str]:
    """Logout and invalidate current session."""
    await auth_service.logout(token.credentials)
    return {"message": "Successfully logged out"}


@router.post("/revoke")
async def revoke_all_sessions(
    auth_service: AuthService = Depends(get_auth_service),
    token: str = Depends(security),
) -> dict[str, str]:
    """Revoke all sessions for the current user."""
    await auth_service.revoke_all(token.credentials)
    return {"message": "All sessions revoked"}


@router.get("/me", response_model=UserResponse)
async def get_current_user(
    auth_service: AuthService = Depends(get_auth_service),
    token: str = Depends(security),
) -> UserResponse:
    """Get current authenticated user info."""
    user = await auth_service.get_current_user(token.credentials)
    if user is None:
        raise HTTPException(
            status_code=status.HTTP_401_UNAUTHORIZED,
            detail="Invalid token",
        )
    return user
"#.to_string()
    }

    fn fastapi_auth_schemas(&self) -> String {
        r#""""
Authentication schemas following OxideKit naming convention (camelCase).
"""
from pydantic import BaseModel, EmailStr


class LoginRequest(BaseModel):
    """Login request payload."""

    email: EmailStr
    password: str


class LoginResponse(BaseModel):
    """Login response with tokens."""

    accessToken: str
    refreshToken: str
    tokenType: str = "Bearer"
    expiresIn: int  # seconds until access token expires


class RefreshRequest(BaseModel):
    """Refresh token request."""

    refreshToken: str


class RefreshResponse(BaseModel):
    """Refresh response with new tokens."""

    accessToken: str
    refreshToken: str
    tokenType: str = "Bearer"
    expiresIn: int


class UserResponse(BaseModel):
    """User info response."""

    id: str
    email: str
    createdAt: str
    updatedAt: str
"#.to_string()
    }

    fn fastapi_auth_service(&self) -> String {
        r#""""
Authentication service implementation.

This is a stub implementation. Replace with your actual auth logic
(database lookups, password verification, etc.)
"""
from datetime import datetime, timedelta, timezone
from typing import Optional

from jose import JWTError, jwt
from passlib.context import CryptContext

from app.core.config import settings
from app.schemas.auth import LoginResponse, RefreshResponse, UserResponse

pwd_context = CryptContext(schemes=["bcrypt"], deprecated="auto")


class AuthService:
    """Authentication service handling login, refresh, and token management."""

    def __init__(self) -> None:
        self.secret_key = settings.SECRET_KEY
        self.algorithm = settings.ALGORITHM
        self.access_token_expire = timedelta(minutes=settings.ACCESS_TOKEN_EXPIRE_MINUTES)
        self.refresh_token_expire = timedelta(days=settings.REFRESH_TOKEN_EXPIRE_DAYS)

    async def login(self, email: str, password: str) -> Optional[LoginResponse]:
        """
        Authenticate user and return tokens.

        TODO: Replace with actual database lookup and password verification.
        """
        # Stub implementation - replace with real auth
        if email == "test@example.com" and password == "password":
            access_token = self._create_token(
                {"sub": "user-123", "email": email, "type": "access"},
                self.access_token_expire,
            )
            refresh_token = self._create_token(
                {"sub": "user-123", "email": email, "type": "refresh"},
                self.refresh_token_expire,
            )
            return LoginResponse(
                accessToken=access_token,
                refreshToken=refresh_token,
                expiresIn=int(self.access_token_expire.total_seconds()),
            )
        return None

    async def refresh(self, refresh_token: str) -> Optional[RefreshResponse]:
        """
        Refresh access token using refresh token.

        Implements token rotation - old refresh token is invalidated.
        TODO: Track refresh tokens in database to detect reuse.
        """
        try:
            payload = jwt.decode(refresh_token, self.secret_key, algorithms=[self.algorithm])
            if payload.get("type") != "refresh":
                return None

            user_id = payload.get("sub")
            email = payload.get("email")

            # Create new tokens
            new_access = self._create_token(
                {"sub": user_id, "email": email, "type": "access"},
                self.access_token_expire,
            )
            new_refresh = self._create_token(
                {"sub": user_id, "email": email, "type": "refresh"},
                self.refresh_token_expire,
            )

            return RefreshResponse(
                accessToken=new_access,
                refreshToken=new_refresh,
                expiresIn=int(self.access_token_expire.total_seconds()),
            )
        except JWTError:
            return None

    async def logout(self, token: str) -> None:
        """
        Logout user by invalidating their current session.

        TODO: Add token to blacklist or revoke in database.
        """
        pass

    async def revoke_all(self, token: str) -> None:
        """
        Revoke all sessions for user.

        TODO: Invalidate all refresh tokens for this user in database.
        """
        pass

    async def get_current_user(self, token: str) -> Optional[UserResponse]:
        """Get current user from access token."""
        try:
            payload = jwt.decode(token, self.secret_key, algorithms=[self.algorithm])
            if payload.get("type") != "access":
                return None

            # Stub response - replace with database lookup
            return UserResponse(
                id=payload.get("sub", ""),
                email=payload.get("email", ""),
                createdAt=datetime.now(timezone.utc).isoformat(),
                updatedAt=datetime.now(timezone.utc).isoformat(),
            )
        except JWTError:
            return None

    def _create_token(self, data: dict, expires_delta: timedelta) -> str:
        """Create a JWT token."""
        to_encode = data.copy()
        expire = datetime.now(timezone.utc) + expires_delta
        to_encode.update({"exp": expire})
        return jwt.encode(to_encode, self.secret_key, algorithm=self.algorithm)
"#.to_string()
    }

    fn fastapi_dockerfile(&self) -> String {
        format!(
            r#"# OxideKit Backend - {name}
# Multi-stage build for production

FROM python:3.11-slim as builder

WORKDIR /app

# Install build dependencies
RUN pip install --no-cache-dir --upgrade pip

# Copy requirements first for caching
COPY requirements.txt .
RUN pip install --no-cache-dir -r requirements.txt

# Production image
FROM python:3.11-slim

WORKDIR /app

# Copy installed packages from builder
COPY --from=builder /usr/local/lib/python3.11/site-packages /usr/local/lib/python3.11/site-packages
COPY --from=builder /usr/local/bin /usr/local/bin

# Copy application code
COPY . .

# Create non-root user
RUN useradd --create-home --shell /bin/bash appuser
USER appuser

# Expose port
EXPOSE {port}

# Health check
HEALTHCHECK --interval=30s --timeout=10s --start-period=5s --retries=3 \
    CMD curl -f http://localhost:{port}/health || exit 1

# Run the application
CMD ["uvicorn", "app.main:app", "--host", "0.0.0.0", "--port", "{port}"]
"#,
            name = self.config.name,
            port = self.config.port
        )
    }

    fn fastapi_docker_compose(&self) -> String {
        format!(
            r#"# Docker Compose for local development
version: "3.8"

services:
  api:
    build:
      context: .
      dockerfile: Dockerfile
    ports:
      - "{port}:{port}"
    environment:
      - DEBUG=true
      - HOST=0.0.0.0
      - PORT={port}
    volumes:
      - .:/app
    command: uvicorn app.main:app --host 0.0.0.0 --port {port} --reload
    healthcheck:
      test: ["CMD", "curl", "-f", "http://localhost:{port}/health"]
      interval: 30s
      timeout: 10s
      retries: 3
      start_period: 10s
"#,
            port = self.config.port
        )
    }

    fn fastapi_env_example(&self) -> String {
        format!(
            r#"# Environment configuration
# Copy to .env and modify for your environment

# Server
DEBUG=false
HOST=0.0.0.0
PORT={port}

# Security - CHANGE THESE IN PRODUCTION
SECRET_KEY=your-super-secret-key-change-this
ACCESS_TOKEN_EXPIRE_MINUTES=15
REFRESH_TOKEN_EXPIRE_DAYS=7

# CORS
CORS_ORIGINS=["http://localhost:3000","http://localhost:5173"]

# Database (if using)
# DATABASE_URL=postgresql://user:password@localhost:5432/dbname
"#,
            port = self.config.port
        )
    }

    fn fastapi_gitignore(&self) -> String {
        r#"# Python
__pycache__/
*.py[cod]
*$py.class
*.so
.Python
build/
develop-eggs/
dist/
downloads/
eggs/
.eggs/
lib/
lib64/
parts/
sdist/
var/
wheels/
*.egg-info/
.installed.cfg
*.egg

# Virtual environments
.env
.venv
env/
venv/
ENV/

# IDE
.idea/
.vscode/
*.swp
*.swo

# Testing
.pytest_cache/
.coverage
htmlcov/
.tox/
.nox/

# mypy
.mypy_cache/
.dmypy.json

# Docker
.docker/

# OS
.DS_Store
Thumbs.db
"#.to_string()
    }

    fn openapi_stub(&self) -> String {
        serde_json::json!({
            "openapi": "3.0.3",
            "info": {
                "title": self.config.name,
                "description": self.config.description,
                "version": "0.1.0"
            },
            "paths": {
                "/health": {
                    "get": {
                        "summary": "Health Check",
                        "operationId": "healthCheck",
                        "responses": {
                            "200": {
                                "description": "Successful Response",
                                "content": {
                                    "application/json": {
                                        "schema": {
                                            "$ref": "#/components/schemas/HealthResponse"
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            },
            "components": {
                "schemas": {
                    "HealthResponse": {
                        "type": "object",
                        "properties": {
                            "status": {
                                "type": "string"
                            },
                            "version": {
                                "type": "string"
                            }
                        },
                        "required": ["status", "version"]
                    }
                }
            }
        }).to_string()
    }

    fn fastapi_conftest(&self) -> String {
        r#""""
Pytest configuration and fixtures.
"""
import pytest
from fastapi.testclient import TestClient

from app.main import app


@pytest.fixture
def client() -> TestClient:
    """Create test client."""
    return TestClient(app)
"#.to_string()
    }

    fn fastapi_test_health(&self) -> String {
        r#""""
Tests for health endpoints.
"""
from fastapi.testclient import TestClient


def test_health_check(client: TestClient) -> None:
    """Test health endpoint returns healthy status."""
    response = client.get("/health")
    assert response.status_code == 200
    data = response.json()
    assert data["status"] == "healthy"
    assert "version" in data


def test_readiness_check(client: TestClient) -> None:
    """Test readiness endpoint."""
    response = client.get("/health/ready")
    assert response.status_code == 200
    assert response.json()["status"] == "ready"


def test_liveness_check(client: TestClient) -> None:
    """Test liveness endpoint."""
    response = client.get("/health/live")
    assert response.status_code == 200
    assert response.json()["status"] == "alive"
"#.to_string()
    }

    fn fastapi_test_auth(&self) -> String {
        r#""""
Tests for auth endpoints.
"""
from fastapi.testclient import TestClient


def test_login_success(client: TestClient) -> None:
    """Test successful login."""
    response = client.post(
        "/auth/login",
        json={"email": "test@example.com", "password": "password"},
    )
    assert response.status_code == 200
    data = response.json()
    assert "accessToken" in data
    assert "refreshToken" in data
    assert data["tokenType"] == "Bearer"


def test_login_invalid_credentials(client: TestClient) -> None:
    """Test login with invalid credentials."""
    response = client.post(
        "/auth/login",
        json={"email": "wrong@example.com", "password": "wrong"},
    )
    assert response.status_code == 401


def test_refresh_token(client: TestClient) -> None:
    """Test token refresh."""
    # First login to get tokens
    login_response = client.post(
        "/auth/login",
        json={"email": "test@example.com", "password": "password"},
    )
    refresh_token = login_response.json()["refreshToken"]

    # Refresh the token
    response = client.post(
        "/auth/refresh",
        json={"refreshToken": refresh_token},
    )
    assert response.status_code == 200
    data = response.json()
    assert "accessToken" in data
    assert "refreshToken" in data


def test_get_current_user(client: TestClient) -> None:
    """Test getting current user info."""
    # First login to get token
    login_response = client.post(
        "/auth/login",
        json={"email": "test@example.com", "password": "password"},
    )
    access_token = login_response.json()["accessToken"]

    # Get user info
    response = client.get(
        "/auth/me",
        headers={"Authorization": f"Bearer {access_token}"},
    )
    assert response.status_code == 200
    data = response.json()
    assert "id" in data
    assert "email" in data
"#.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_project_config_creation() {
        let config = ProjectConfig::new("test-api");
        assert_eq!(config.name, "test-api");
        assert!(matches!(config.stack, BackendStack::FastApi));
        assert!(config.include_auth);
    }

    #[test]
    fn test_project_config_builder_pattern() {
        let config = ProjectConfig::new("my-api")
            .with_description("My awesome API")
            .with_stack(BackendStack::FastApi)
            .with_auth(true);

        assert_eq!(config.name, "my-api");
        assert_eq!(config.description, "My awesome API");
        assert!(config.include_auth);
    }

    #[test]
    fn test_backend_stack_display_name() {
        assert_eq!(BackendStack::FastApi.display_name(), "FastAPI");
        assert_eq!(BackendStack::Axum.display_name(), "Axum");
        assert_eq!(BackendStack::Actix.display_name(), "Actix-web");
    }

    #[test]
    fn test_backend_stack_supported() {
        assert!(BackendStack::FastApi.is_supported());
        assert!(!BackendStack::Axum.is_supported());
        assert!(!BackendStack::Actix.is_supported());
    }

    #[test]
    fn test_scaffold_generates_fastapi_project() {
        let config = ProjectConfig::new("test-api");
        let scaffold = ProjectScaffold::new(config);
        let temp_dir = TempDir::new().unwrap();

        let result = scaffold.generate(temp_dir.path());
        assert!(result.is_ok());

        let output = result.unwrap();
        assert!(output.join("app/main.py").exists());
        assert!(output.join("Dockerfile").exists());
        assert!(output.join("docker-compose.yml").exists());
        assert!(output.join("openapi.json").exists());
    }

    #[test]
    fn test_scaffold_includes_auth_when_enabled() {
        let config = ProjectConfig::new("test-api").with_auth(true);
        let scaffold = ProjectScaffold::new(config);
        let temp_dir = TempDir::new().unwrap();

        let output = scaffold.generate(temp_dir.path()).unwrap();
        assert!(output.join("app/api/v1/auth.py").exists());
        assert!(output.join("app/schemas/auth.py").exists());
        assert!(output.join("app/services/auth.py").exists());
    }

    #[test]
    fn test_scaffold_excludes_auth_when_disabled() {
        let config = ProjectConfig::new("test-api").with_auth(false);
        let scaffold = ProjectScaffold::new(config);
        let temp_dir = TempDir::new().unwrap();

        let output = scaffold.generate(temp_dir.path()).unwrap();
        assert!(!output.join("app/api/v1/auth.py").exists());
    }

    #[test]
    fn test_unsupported_stack_returns_error() {
        let config = ProjectConfig::new("test-api").with_stack(BackendStack::Axum);
        let scaffold = ProjectScaffold::new(config);
        let temp_dir = TempDir::new().unwrap();

        let result = scaffold.generate(temp_dir.path());
        assert!(result.is_err());
    }
}
