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

## Commands

`.riotid` - Set your RiotId i.e. `.riotid Martige#NA1` (required before joining queue)

`.maps` - Lists all maps available for map vote


_These are commands used during the `.start` process:_

`.defense` - An option to pick the defense side after the draft (if you are Captain B)

`.attack` - An option to pick the attack side after the draft (if you are Captain B)

### Admin Commands - restricted to an 'admin' role if provided in config

`.setup` - Start the match setup process

`.addmap` - Add a map to the map vote i.e. `.addmap mapname`

`.removemap` - Remove a map from the map vote i.e. `.removemap mapname`

`.cancel` - Cancels `.start` process & retains current queue
