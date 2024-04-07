pub const KEYSPACE: &'static str = "CREATE KEYSPACE IF NOT EXISTS tutorial
  WITH REPLICATION = { 
   'class' : 'SimpleStrategy', 
   'replication_factor' : 1 
  }";

pub const MESSAGES_TABLE: &'static str = "CREATE TABLE IF NOT EXISTS tutorial.messages (
    message_id uuid,
    name text,
    message text,
    PRIMARY KEY (message_id)
   ) with compaction = { 'class' : 'LeveledCompactionStrategy' }
    and cdc = {'enabled': true}";
