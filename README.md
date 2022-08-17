# tinyKV

## Database Migration
```sh
# init migration
sea migrate init
# create new migration
sea migrate generate create_x_table 
# generate entity
sea generate entity -u sqlite://db.sqlite3 -o src/entity -v
