# SECURITY_NOTES

## API Key Protection

- Mobile clients must not store LLM provider API keys.
- API keys belong on the server-side or in the Agent Gateway, not in the client application.
- Device tokens are not provider API keys.

## Secrets Management

- Secrets must not be committed to version control.
- Environment files (.env, .env.*) are excluded via `.gitignore`.
- Private keys (*.pem, *.key) must never be committed.

## Local Data Privacy

- Local data may contain private daily notes, personal context, habits, and schedules.
- SQLite database files (*.sqlite, *.sqlite3, *.db) are excluded from version control.

## Future Sync Considerations

- Future sync must consider encryption for data in transit and at rest.
- Device authentication and device tokens will be required.
- User consent must be obtained before data leaves the device.

## AI Request Privacy

- AI requests must avoid sending unnecessary private context to LLM providers.
- The Agent Gateway should filter or minimize personal data before forwarding to LLM providers.
