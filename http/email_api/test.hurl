POST https://api.mailersend.com/v1/email
Authorization: Bearer {{token}}
{
	"from": {
		"email": "hello@{{from_domain}}",
		"name": "Test"
	},
	"to": [
		{
			"email": "{{to_address}}",
			"name": "John Mailer"
		}
	],
	"subject": "Hello from Test!",
	"text": "This is just a friendly hello from your friends at Test.",
	"html": "<b>This is just a friendly hello from your friends at Test.</b>"
}
