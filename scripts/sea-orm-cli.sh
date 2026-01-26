cargo install sea-orm-cli@^2.0.0-rc
rustup component add rustfmt
export DATABASE_URL=mysql://moyanxinxu:moyanxinxu@uav-database:3306/uav
sea-orm-cli generate entity -s uav --with-serde both --model-extra-attributes 'serde(rename_all = "camelCase") ' --date-time-crate chrono -o ./src/entity