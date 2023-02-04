# gitevents-discord-bot

WIP: it should offer a modal to a discord user and create an issue with github with the data received from the modal.

Currently discord throws "This interaction failed".

## Setup application_command

curl -X POST -H "Authorization: Bot <DISCORD_BOT_TOKEN>" -H "Content-Type: application/json" -d '{"name": "new_event", "type_value": 1, "description": "Create a new event on GitEvents"}' https://discord.com/api/v10/applications/<DISCORD_APPLICATION_ID/commands