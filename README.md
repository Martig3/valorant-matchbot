# valorant-matchbot

Simple Discord bot for managing & organizing a queue for 10 man scrims in Valorant

## Features
// TODO
### Example Screenshots
// TODO

No CI/CD yet so clone the repo, create a `config.yaml` file (see example below) and run using standard `cargo run`

**Note:** Make sure to only allow the bot to listen/read messages in one channel only. 
### Example config.yaml

```yaml
post-setup-msg: GLHF! Add any string here -- optional
discord:
  token: <your discord bot api token>
  admin_role_id: <a discord server role id> -- optional, but highly recommended!!!
  assign_role_id: <a dicord role id to assign for user on queue join> -- optional
```
