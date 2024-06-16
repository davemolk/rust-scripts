not meant for concurrent use and not safe for concurrent use

```
kvs list                                       list all keys in db
kvs get       <key>                            get the value for given key
kvs set       <key>    <value>                 set a value for a given key, overwrites any existing value(s)
kvs setk      ...<key> <value>                 set a value to multiple keys, appending in each case
kvs setv      <key>    <value>                 append a new value to the given key
kvs update    <key>    <new_key>               update a key name
kvs update    <key>    <value>    <new_value>  update a value for a key
kvs duplicate <key>    <new_key>               copy a key's values to a new key (old key and value remain unchanged)
kvs remove    <key>    <value>                 removes a value from a key
kvs delete    <key>                            deletes a key and its value(s)
kvs backup    <new_file_name>                  makes a copy of the current db file
kvs undo                                       undo the last operation (supported for set, setk, setv, update, duplicate, remove, and delete)

kvs help                                       prints usage
```