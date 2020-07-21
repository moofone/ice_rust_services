module.exports = {
  apps: [
    {
      name: 'dpplns',
      script: '/icedev/ice_rust_services/target/release/dpplns --time',
      instances: 1,
      autorestart: true,
      watch: false,
      time: true,
    },
    {
      name: 'blocks_nats_to_sql',
      script: '/icedev/ice_rust_services/target/release/blocks-nats-to-sql --time',
      instances: 1,
      autorestart: true,
      watch: false,
      time: true,

    },
    {
      name: 'event_scheduler',
      script: '/icedev/ice_rust_services/target/release/event-scheduler --time',
      instances: 1,
      autorestart: true,
      watch: false,
      time: true,

    },
    {
      name: 'shares_to_pg',
      script: '/icedev/ice_rust_services/target/release/shares-to-pg --time',
      instances: 1,
      autorestart: true,
      watch: false,
      time: true,

    },
    // {
    //   name: 'kda_block_conf',
    //   script: '/icedev/datahub/services/kda-blockconf/index',
    //   instances: 1,
    //   autorestart: true,
    //   watch: false,
    //   time: true,

    // },
  ]



};
