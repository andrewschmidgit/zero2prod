POST {{host}}/subscriptions
[FormParams]
name: John Doe
email: {{newUuid}}@test.com
HTTP 200
[Captures]
token: body

GET {{host}}/subscriptions/confirm
[QueryStringParams]
subscription_token: {{token}}
HTTP 200
