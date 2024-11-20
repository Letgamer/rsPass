# rsPass Backend 🦀
![GitHub commit activity](https://img.shields.io/github/commit-activity/w/Letgamer/rsPass)
![GitHub top language](https://img.shields.io/github/languages/top/Letgamer/rsPass)
![GitHub Actions Workflow Status](https://img.shields.io/github/actions/workflow/status/Letgamer/rsPass/.github%2Fworkflows%2Fdocker-publish.yml)
![GitHub repo size](https://img.shields.io/github/repo-size/Letgamer/rsPass) 
[![Docker](https://badgen.net/badge/icon/docker?icon=docker&label)](https://https://docker.com/)
[![Open Source Love svg1](https://badges.frapsoft.com/os/v1/open-source.svg?v=103)](https://github.com/ellerbrock/open-source-badges/)
[![HitCount](https://hits.dwyl.com/Letgamer/rsPass.svg?style=flat-square)](http://hits.dwyl.com/Letgamer/rsPass)




rsPass is a modern, secure password manager backend written in Rust with a focus on performance, security, and scalability. 🚀   
It provides a REST API to manage user authentication, password storage, and secure data synchronization. 📚  
rsPass leverages SQLite for data storage, JWT for authentication, and integrates with Swagger-UI for API documentation. ✨

## Features
- User Management: Secure registration, login, and password change.  
- Data Encryption: Stores user-specific data in an encrypted format.  
- JWT Authentication: Stateless and secure authentication mechanism.  
- Swagger-UI Integration: Built-in API documentation.  
- Environment Configuration: Flexible setup using environment variables.  
- Database Transactions: Ensures data consistency using rusqlite transactions.  
- Security by Design: Input validation, SQL injection prevention, and Argon2id for password hashing.  

## Technology Stack
- Rust: The programming language powering rsPass.  
- actix-web: Web framework for building the REST API.  
- rusqlite: Lightweight SQLite library for database operations.  
- JWT: Token-based authentication using actix-web-httpauth.  
- utoipa: API documentation generation.  
- Swagger-UI: Interactive API explorer.  

## Development
### Prerequisites
- Rust (latest stable version)
- Docker (optional, for containerized setup)
### Setup
Clone the repository:
```
git clone https://github.com/yourusername/rspass-backend.git
cd rspass-backend
```
Set up the environment variables: Create a .env file in the root directory:
```
JWT_SECRET=your_jwt_secret_key
```
Run the programm:
```
cargo watch -x run
```

## API Documentation
rsPass integrates with Swagger-UI for API documentation.  
After starting the server, navigate to https://<DOMAIN>/swagger-ui to explore the available endpoints.

## Deployment
The Backend can be deployed using Docker:
```
docker pull ghcr.io/letgamer/rspass:main
```
A docker-compose example file can also be found at [`docker-compose.yaml`](https://github.com/Letgamer/rsPass/blob/main/docker-compose.yaml).

## Routes
See the Wiki for detailed Documentation about the Routes and API Logic
