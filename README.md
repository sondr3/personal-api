<h1 align="center">Personal API</h1>
<p align="center">
    <a href="https://github.com/sondr3/personal-api/actions"><img alt="GitHub Actions Status" src="https://github.com/sondr3/personal-api/workflows/pipeline/badge.svg" /></a>
</p>

<p align="center">
    <b>A REST API for myself.</b>
</p>

- **What**: I use it for my website, mostly.
- **Why**: Rust or bust.

<details>
<summary>Table of Contents</summary>
<br />

## Table of Contents

- [Installation](#installation)
- [Getting started](#getting-started)
- [License](#license)
</details>

# Installation

Clone the repo, copy `.env.example` to `.env` and do a `cargo run`.

# Getting started

You probably shouldn't, but hey. You need to configure the environment 
variables, I'll briefly explain them:

```dotenv
LOGIN=username                 # GitHub username
TOKEN=token                    # GitHub token
WHOAMI=name                    # Used for simple spam protection for the contact form
CONTACT_EMAIL=mail@example.org # The email adress that contact form emails are sent from
EMAIL=mail@example.org         # Your email adress
RELAY=smtp.example.org         # The SMTP relay URL
SMTP_USER=user                 # SMTP username
SMTP_PASS=pass                 # SMTP password
POSTGRES_DB=personal           # Database name
POSTGRES_USER=sondre           # Database username
POSTGRES_PASSWORD=password     # Database password
POSTGRES_HOST=localhost        # Database host
DATABASE_URL=postgresql://sondre:password@localhost:5432/personal?schema=public # Used to connect via API
```

**NOTE:** The `url` should be `localhost` in the `DATABASE_URL` when using the 
`docker-compose.local.yml` file and then `personal_db` when using `docker-compose.yml`.

# License

MIT
