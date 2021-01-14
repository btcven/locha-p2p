var searchIndex = JSON.parse('{\
"locha_p2p":{"doc":"Locha P2PThis library contains the necessary code to set…","i":[[4,"Protocol","locha_p2p","`Protocol` describes all possible multiaddress protocols.",null,null],[13,"Dccp","","",0,null],[13,"Dns","","",0,null],[13,"Dns4","","",0,null],[13,"Dns6","","",0,null],[13,"Dnsaddr","","",0,null],[13,"Http","","",0,null],[13,"Https","","",0,null],[13,"Ip4","","",0,null],[13,"Ip6","","",0,null],[13,"P2pWebRtcDirect","","",0,null],[13,"P2pWebRtcStar","","",0,null],[13,"P2pWebSocketStar","","",0,null],[13,"Memory","","Contains the \\\"port\\\" to contact. Similar to TCP or UDP, 0…",0,null],[13,"Onion","","",0,null],[13,"Onion3","","",0,null],[13,"P2p","","",0,null],[13,"P2pCircuit","","",0,null],[13,"Quic","","",0,null],[13,"Sctp","","",0,null],[13,"Tcp","","",0,null],[13,"Udp","","",0,null],[13,"Udt","","",0,null],[13,"Unix","","",0,null],[13,"Utp","","",0,null],[13,"Ws","","",0,null],[13,"Wss","","",0,null],[3,"Multiaddr","","Representation of a Multiaddr.",null,null],[3,"PeerId","","Identifier of a peer of the network.",null,null],[5,"build_transport","","Builds the `Transport` used in Locha P2P",null,[[["keypair",4]],[["boxed",3],["result",6]]]],[0,"identity","","IdentityAn identity is composed from a secp256k1 ECDSA…",null,null],[3,"Identity","locha_p2p::identity","The identity of a single node.",null,null],[17,"SECRET_KEY_LENGTH","","Length of a secp256k1 Secret Key in bytes",null,null],[11,"generate","","Generate a random identity.",1,[[],["identity",3]]],[11,"to_file","","Save `Identity` to a file",1,[[["asref",8],["path",3]],["result",6]]],[11,"from_file","","Load `Identity` from file",1,[[["asref",8],["path",3]],[["result",6],["identity",3]]]],[11,"id","","Get the PeerId from this `Identity`.",1,[[],["peerid",3]]],[11,"keypair","","Get the Keypair from this `Identity`.",1,[[],["keypair",4]]],[0,"p2p","locha_p2p","",null,null],[5,"build_swarm","locha_p2p::p2p","Builds a swarm for Locha P2P",null,[[["identity",3]],[["result",4],["error",3]]]],[0,"behaviour","","Locha libp2p behaviour",null,null],[3,"Behaviour","locha_p2p::p2p::behaviour","",null,null],[4,"BehaviourEvent","","A behaviour event",null,null],[13,"Message","","A new message received from a peer",2,null],[5,"bootstrap_nodes","","Bootstrap nodes",null,[[],["vec",3]]],[6,"BehaviourEventStream","","Stream of BehaviourEvents",null,null],[17,"AGENT_VERSION","","Node/agent version string.",null,null],[17,"LOCHA_KAD_PROTOCOL_NAME","","Locha P2P Kademlia protocol string",null,null],[17,"BOOTSTRAP_NODES","","Default bootstrap nodes",null,null],[11,"new","","",3,[[["identity",3]]]],[11,"add_peer","","Add a peer to the network behaviour.",3,[[["peerid",3],["multiaddr",3]]]],[11,"subscribe","","",3,[[["topic",3]]]],[11,"publish","","",3,[[["topic",3]],[["result",4],["publisherror",4]]]],[11,"bootstrap","","Start bootstrap process",3,[[]]],[11,"kademlia","","",3,[[],["kademlia",3]]],[6,"Swarm","locha_p2p::p2p","Locha P2P swarm",null,null],[0,"runtime","locha_p2p","RuntimeThis contains a runtime for the Chat. The network…",null,null],[3,"NetworkInfo","locha_p2p::runtime","Information about the network obtained by…",null,null],[12,"num_peers","","The total number of connected peers.",4,null],[12,"num_connections","","The total number of connections, both established and…",4,null],[12,"num_connections_pending","","The total number of pending connections, both incoming and…",4,null],[12,"num_connections_established","","The total number of established connections.",4,null],[3,"EntryView","","A cloned, immutable view of an entry that is either…",null,null],[12,"node","","The node represented by the entry.",5,null],[12,"status","","The status of the node.",5,null],[3,"Key","","A `Key` in the DHT keyspace with preserved preimage.",null,null],[3,"Addresses","","A non-empty list of (unique) addresses of a peer in the…",null,null],[3,"Runtime","","Locha P2P runtime",null,null],[0,"config","","Runtime configuration",null,null],[3,"RuntimeConfig","locha_p2p::runtime::config","Runtime configuration",null,null],[12,"identity","","",6,null],[12,"listen_addr","","",6,null],[12,"channel_cap","","",6,null],[12,"heartbeat_interval","","",6,null],[12,"mdns","","",6,null],[12,"upnp","","",6,null],[12,"bootstrap_nodes","","",6,null],[0,"error","locha_p2p::runtime","Runtime errors",null,null],[4,"Error","locha_p2p::runtime::error","Chat service error",null,null],[13,"Io","","I/O error",7,null],[13,"TransportError","","libp2p transport error",7,null],[13,"AlreadyStarted","","Chat service is already started",7,null],[13,"NotStarted","","Chat service has not been started",7,null],[13,"ChannelClosed","","Chat service channel is closed",7,null],[0,"events","locha_p2p::runtime","Runtime eventsThese are the network events reported during…",null,null],[3,"RuntimeEventsLogger","locha_p2p::runtime::events","Small wrapper around [`RuntimeEvents`] that logs all events.",null,null],[8,"RuntimeEvents","","Chat service events",null,null],[11,"on_new_message","","New message received",8,[[["peerid",3],["vec",3]]]],[11,"on_connection_established","","Connection established to the given peer.",8,[[["nonzerou32",3],["peerid",3],["connectedpoint",4]]]],[11,"on_connection_closed","","Connection closed with the given peer.",8,[[["peerid",3],["string",3],["option",4],["connectedpoint",4]]]],[11,"on_incomming_connection","","New incoming connection",8,[[["multiaddr",3]]]],[11,"on_incomming_connection_error","","Incoming connection failed",8,[[["pendingconnectionerror",4],["multiaddr",3]]]],[11,"on_banned_peer","","Peer on the given endpoint was banned.",8,[[["peerid",3],["connectedpoint",4]]]],[11,"on_unreachable_addr","","Unreachable peer address",8,[[["pendingconnectionerror",4],["peerid",3],["multiaddr",3]]]],[11,"on_unknown_peer_unreachable_addr","","Unknown peer is unreachable",8,[[["pendingconnectionerror",4],["multiaddr",3]]]],[11,"on_new_listen_addr","","New listening address",8,[[["multiaddr",3]]]],[11,"on_expired_listen_addr","","A listening address expired",8,[[["multiaddr",3]]]],[11,"on_listener_closed","","Listener closed.",8,[[["result",4]]]],[11,"on_listener_error","","An error ocurred on a listener.",8,[[["error",3]]]],[11,"on_dialing","","When dialing the given peer.",8,[[["peerid",3]]]],[11,"new","","Create a new [`RuntimeEventsLogger`]",9,[[]]],[6,"Kbucket","locha_p2p::runtime","An owned Kademlia K-bucket",null,null],[11,"new","","Create a runtime for Locha P2P",10,[[["runtimeconfig",3],["runtimeevents",8],["box",3]],[["result",4],["error",4]]]],[11,"bootstrap","","Start bootstrapping",10,[[]]],[11,"kbuckets","","",10,[[]]],[11,"network_info","","",10,[[]]],[11,"stop","","Stop the runtime.",10,[[]]],[11,"dial","","Dial a peer using it\'s multiaddress",10,[[["multiaddr",3]]]],[11,"send_message","","Send a message",10,[[["vec",3]]]],[11,"external_addresses","","",10,[[]]],[11,"peer_id","","",10,[[]]],[0,"upnp","locha_p2p","UPnPThis is the Universal Plug and Play (UPnP)…",null,null],[3,"UpnpBehaviour","locha_p2p::upnp","",null,null],[18,"INTERVAL","","Interval used for checking External IPv4 address and Port…",11,null],[11,"new","","Create a new [`UpnpBehaviour`]",11,[[["cow",4]],["upnpbehaviour",3]]],[0,"msg","locha_p2p","",null,null],[3,"Msg","locha_p2p::msg","",null,null],[12,"text","","",12,null],[12,"type_file","","",12,null],[12,"file","","",12,null],[3,"MessageData","","",null,null],[12,"from_uid","","",13,null],[12,"to_uid","","",13,null],[12,"msg_id","","",13,null],[12,"timestamp","","",13,null],[12,"shipping_time","","",13,null],[12,"type","","",13,null],[12,"msg","","",13,null],[0,"items","","",null,null],[3,"Content","locha_p2p::msg::items","",null,null],[12,"to_uid","","",14,null],[12,"msg_id","","",14,null],[12,"timestamp","","",14,null],[12,"shipping_time","","",14,null],[12,"type_message","","",14,null],[12,"text","","",14,null],[12,"type_file","","",14,null],[12,"file","","",14,null],[12,"status","","",14,null],[12,"from_uid","","",14,null],[11,"from","locha_p2p","",0,[[]]],[11,"into","","",0,[[]]],[11,"to_owned","","",0,[[]]],[11,"clone_into","","",0,[[]]],[11,"to_string","","",0,[[],["string",3]]],[11,"borrow","","",0,[[]]],[11,"borrow_mut","","",0,[[]]],[11,"try_from","","",0,[[],["result",4]]],[11,"try_into","","",0,[[],["result",4]]],[11,"type_id","","",0,[[],["typeid",3]]],[11,"vzip","","",0,[[]]],[11,"from","","",15,[[]]],[11,"into","","",15,[[]]],[11,"to_owned","","",15,[[]]],[11,"clone_into","","",15,[[]]],[11,"to_string","","",15,[[],["string",3]]],[11,"borrow","","",15,[[]]],[11,"borrow_mut","","",15,[[]]],[11,"try_from","","",15,[[],["result",4]]],[11,"try_into","","",15,[[],["result",4]]],[11,"type_id","","",15,[[],["typeid",3]]],[11,"protocol_name","","",15,[[]]],[11,"vzip","","",15,[[]]],[11,"get_hash","","",15,[[]]],[11,"from","","",16,[[]]],[11,"into","","",16,[[]]],[11,"to_owned","","",16,[[]]],[11,"clone_into","","",16,[[]]],[11,"to_string","","",16,[[],["string",3]]],[11,"borrow","","",16,[[]]],[11,"borrow_mut","","",16,[[]]],[11,"try_from","","",16,[[],["result",4]]],[11,"try_into","","",16,[[],["result",4]]],[11,"type_id","","",16,[[],["typeid",3]]],[11,"protocol_name","","",16,[[]]],[11,"vzip","","",16,[[]]],[11,"get_hash","","",16,[[]]],[11,"from","locha_p2p::identity","",1,[[]]],[11,"into","","",1,[[]]],[11,"to_owned","","",1,[[]]],[11,"clone_into","","",1,[[]]],[11,"borrow","","",1,[[]]],[11,"borrow_mut","","",1,[[]]],[11,"try_from","","",1,[[],["result",4]]],[11,"try_into","","",1,[[],["result",4]]],[11,"type_id","","",1,[[],["typeid",3]]],[11,"vzip","","",1,[[]]],[11,"from","locha_p2p::p2p::behaviour","",3,[[]]],[11,"into","","",3,[[]]],[11,"borrow","","",3,[[]]],[11,"borrow_mut","","",3,[[]]],[11,"try_from","","",3,[[],["result",4]]],[11,"try_into","","",3,[[],["result",4]]],[11,"type_id","","",3,[[],["typeid",3]]],[11,"vzip","","",3,[[]]],[11,"from","","",2,[[]]],[11,"into","","",2,[[]]],[11,"borrow","","",2,[[]]],[11,"borrow_mut","","",2,[[]]],[11,"try_from","","",2,[[],["result",4]]],[11,"try_into","","",2,[[],["result",4]]],[11,"type_id","","",2,[[],["typeid",3]]],[11,"vzip","","",2,[[]]],[11,"from","locha_p2p::runtime","",4,[[]]],[11,"into","","",4,[[]]],[11,"to_owned","","",4,[[]]],[11,"clone_into","","",4,[[]]],[11,"borrow","","",4,[[]]],[11,"borrow_mut","","",4,[[]]],[11,"try_from","","",4,[[],["result",4]]],[11,"try_into","","",4,[[],["result",4]]],[11,"type_id","","",4,[[],["typeid",3]]],[11,"vzip","","",4,[[]]],[11,"from","","",5,[[]]],[11,"into","","",5,[[]]],[11,"to_owned","","",5,[[]]],[11,"clone_into","","",5,[[]]],[11,"borrow","","",5,[[]]],[11,"borrow_mut","","",5,[[]]],[11,"try_from","","",5,[[],["result",4]]],[11,"try_into","","",5,[[],["result",4]]],[11,"type_id","","",5,[[],["typeid",3]]],[11,"vzip","","",5,[[]]],[11,"from","","",17,[[]]],[11,"into","","",17,[[]]],[11,"to_owned","","",17,[[]]],[11,"clone_into","","",17,[[]]],[11,"borrow","","",17,[[]]],[11,"borrow_mut","","",17,[[]]],[11,"try_from","","",17,[[],["result",4]]],[11,"try_into","","",17,[[],["result",4]]],[11,"type_id","","",17,[[],["typeid",3]]],[11,"vzip","","",17,[[]]],[11,"get_hash","","",17,[[]]],[11,"from","","",18,[[]]],[11,"into","","",18,[[]]],[11,"to_owned","","",18,[[]]],[11,"clone_into","","",18,[[]]],[11,"borrow","","",18,[[]]],[11,"borrow_mut","","",18,[[]]],[11,"try_from","","",18,[[],["result",4]]],[11,"try_into","","",18,[[],["result",4]]],[11,"type_id","","",18,[[],["typeid",3]]],[11,"vzip","","",18,[[]]],[11,"from","","",10,[[]]],[11,"into","","",10,[[]]],[11,"to_owned","","",10,[[]]],[11,"clone_into","","",10,[[]]],[11,"borrow","","",10,[[]]],[11,"borrow_mut","","",10,[[]]],[11,"try_from","","",10,[[],["result",4]]],[11,"try_into","","",10,[[],["result",4]]],[11,"type_id","","",10,[[],["typeid",3]]],[11,"vzip","","",10,[[]]],[11,"from","locha_p2p::runtime::config","",6,[[]]],[11,"into","","",6,[[]]],[11,"borrow","","",6,[[]]],[11,"borrow_mut","","",6,[[]]],[11,"try_from","","",6,[[],["result",4]]],[11,"try_into","","",6,[[],["result",4]]],[11,"type_id","","",6,[[],["typeid",3]]],[11,"vzip","","",6,[[]]],[11,"from","locha_p2p::runtime::error","",7,[[]]],[11,"into","","",7,[[]]],[11,"to_string","","",7,[[],["string",3]]],[11,"borrow","","",7,[[]]],[11,"borrow_mut","","",7,[[]]],[11,"try_from","","",7,[[],["result",4]]],[11,"try_into","","",7,[[],["result",4]]],[11,"type_id","","",7,[[],["typeid",3]]],[11,"vzip","","",7,[[]]],[11,"from","locha_p2p::runtime::events","",9,[[]]],[11,"into","","",9,[[]]],[11,"borrow","","",9,[[]]],[11,"borrow_mut","","",9,[[]]],[11,"try_from","","",9,[[],["result",4]]],[11,"try_into","","",9,[[],["result",4]]],[11,"type_id","","",9,[[],["typeid",3]]],[11,"vzip","","",9,[[]]],[11,"from","locha_p2p::upnp","",11,[[]]],[11,"into","","",11,[[]]],[11,"borrow","","",11,[[]]],[11,"borrow_mut","","",11,[[]]],[11,"try_from","","",11,[[],["result",4]]],[11,"try_into","","",11,[[],["result",4]]],[11,"type_id","","",11,[[],["typeid",3]]],[11,"vzip","","",11,[[]]],[11,"from","locha_p2p::msg","",12,[[]]],[11,"into","","",12,[[]]],[11,"borrow","","",12,[[]]],[11,"borrow_mut","","",12,[[]]],[11,"try_from","","",12,[[],["result",4]]],[11,"try_into","","",12,[[],["result",4]]],[11,"type_id","","",12,[[],["typeid",3]]],[11,"vzip","","",12,[[]]],[11,"from","","",13,[[]]],[11,"into","","",13,[[]]],[11,"borrow","","",13,[[]]],[11,"borrow_mut","","",13,[[]]],[11,"try_from","","",13,[[],["result",4]]],[11,"try_into","","",13,[[],["result",4]]],[11,"type_id","","",13,[[],["typeid",3]]],[11,"vzip","","",13,[[]]],[11,"from","locha_p2p::msg::items","",14,[[]]],[11,"into","","",14,[[]]],[11,"to_owned","","",14,[[]]],[11,"clone_into","","",14,[[]]],[11,"borrow","","",14,[[]]],[11,"borrow_mut","","",14,[[]]],[11,"try_from","","",14,[[],["result",4]]],[11,"try_into","","",14,[[],["result",4]]],[11,"type_id","","",14,[[],["typeid",3]]],[11,"vzip","","",14,[[]]],[11,"clone","locha_p2p","",0,[[],["protocol",4]]],[11,"clone","","",15,[[],["multiaddr",3]]],[11,"hash","","",15,[[]]],[11,"serialize","","",15,[[],["result",4]]],[11,"from","","",15,[[["ipv6addr",3]],["multiaddr",3]]],[11,"from","","",0,[[["ipaddr",4]],["protocol",4]]],[11,"from","","",15,[[["ipaddr",4]],["multiaddr",3]]],[11,"from","","",15,[[["protocol",4]],["multiaddr",3]]],[11,"from","","",0,[[["ipv6addr",3]],["protocol",4]]],[11,"from","","",0,[[["ipv4addr",3]],["protocol",4]]],[11,"from","","",15,[[["ipv4addr",3]],["multiaddr",3]]],[11,"from_iter","","",15,[[],["multiaddr",3]]],[11,"as_ref","","",15,[[]]],[11,"cmp","","",15,[[["multiaddr",3]],["ordering",4]]],[11,"fmt","","",15,[[["formatter",3]],[["error",3],["result",4]]]],[11,"fmt","","",0,[[["formatter",3]],[["error",3],["result",4]]]],[11,"deserialize","","",15,[[],[["result",4],["multiaddr",3]]]],[11,"eq","","",0,[[["protocol",4]]]],[11,"ne","","",0,[[["protocol",4]]]],[11,"eq","","",15,[[["multiaddr",3]]]],[11,"ne","","",15,[[["multiaddr",3]]]],[11,"partial_cmp","","",15,[[["multiaddr",3]],[["ordering",4],["option",4]]]],[11,"lt","","",15,[[["multiaddr",3]]]],[11,"le","","",15,[[["multiaddr",3]]]],[11,"gt","","",15,[[["multiaddr",3]]]],[11,"ge","","",15,[[["multiaddr",3]]]],[11,"fmt","","",0,[[["formatter",3]],[["error",3],["result",4]]]],[11,"fmt","","Convert a Multiaddr to a string",15,[[["formatter",3]],[["error",3],["result",4]]]],[11,"from_str","","",15,[[],[["multiaddr",3],["error",4],["result",4]]]],[11,"try_from","","",15,[[],[["multiaddr",3],["error",4],["result",4]]]],[11,"try_from","","",15,[[["string",3]],[["multiaddr",3],["error",4],["result",4]]]],[11,"try_from","","",15,[[["vec",3]],[["multiaddr",3],["error",4],["result",4]]]],[11,"fmt","locha_p2p::runtime","",4,[[["formatter",3]],[["error",3],["result",4]]]],[11,"fmt","locha_p2p","",16,[[["formatter",3]],[["error",3],["result",4]]]],[11,"cmp","","",16,[[["peerid",3]],["ordering",4]]],[11,"eq","","",16,[[["peerid",3]]]],[11,"clone","locha_p2p::runtime","",4,[[],["networkinfo",3]]],[11,"clone","locha_p2p","",16,[[],["peerid",3]]],[11,"hash","","",16,[[]]],[11,"peer_id","","",16,[[],["peerid",3]]],[11,"as_ref","","",16,[[]]],[11,"from_str","","",16,[[],[["result",4],["peerid",3]]]],[11,"borrow","","",16,[[]]],[11,"try_from","","",16,[[["code",4],["multihashgeneric",3]],[["result",4],["peerid",3]]]],[11,"try_from","","",16,[[["vec",3]],[["result",4],["peerid",3]]]],[11,"from","","",16,[[["publickey",4]],["peerid",3]]],[11,"partial_cmp","","",16,[[["peerid",3]],[["ordering",4],["option",4]]]],[11,"fmt","","",16,[[["formatter",3]],[["error",3],["result",4]]]],[11,"fmt","locha_p2p::runtime","",18,[[["formatter",3]],[["error",3],["result",4]]]],[11,"fmt","","",5,[[["formatter",3]],[["error",3],["result",4]]]],[11,"fmt","","",17,[[["formatter",3]],[["error",3],["result",4]]]],[11,"eq","","",17,[[["key",3]]]],[11,"clone","","",5,[[],["entryview",3]]],[11,"clone","","",17,[[],["key",3]]],[11,"clone","","",18,[[],["addresses",3]]],[11,"hash","","",17,[[]]],[11,"from","","",17,[[["code",4],["multihashgeneric",3]],[["key",3],["multihashgeneric",3]]]],[11,"from","","",17,[[["peerid",3]],[["key",3],["peerid",3]]]],[11,"into","","",17,[[],["keybytes",3]]],[11,"as_ref","","",17,[[],["keybytes",3]]],[11,"as_ref","","",5,[[],["keybytes",3]]],[11,"on_new_message","locha_p2p::runtime::events","",9,[[["peerid",3],["vec",3]]]],[11,"on_connection_established","","",9,[[["nonzerou32",3],["peerid",3],["connectedpoint",4]]]],[11,"on_connection_closed","","",9,[[["peerid",3],["string",3],["option",4],["connectedpoint",4]]]],[11,"on_incomming_connection","","",9,[[["multiaddr",3]]]],[11,"on_incomming_connection_error","","",9,[[["pendingconnectionerror",4],["multiaddr",3]]]],[11,"on_banned_peer","","",9,[[["peerid",3],["connectedpoint",4]]]],[11,"on_unreachable_addr","","",9,[[["pendingconnectionerror",4],["peerid",3],["multiaddr",3]]]],[11,"on_unknown_peer_unreachable_addr","","",9,[[["pendingconnectionerror",4],["multiaddr",3]]]],[11,"on_new_listen_addr","","",9,[[["multiaddr",3]]]],[11,"on_expired_listen_addr","","",9,[[["multiaddr",3]]]],[11,"on_listener_closed","","",9,[[["result",4]]]],[11,"on_listener_error","","",9,[[["error",3]]]],[11,"on_dialing","","",9,[[["peerid",3]]]],[11,"from","locha_p2p::identity","Generate an identity from a secp256k1 secret key.",1,[[["secretkey",3]],["identity",3]]],[11,"from","locha_p2p::runtime::error","",7,[[["error",3]],["error",4]]],[11,"from","","",7,[[["error",3],["transporterror",4]],["error",4]]],[11,"clone","locha_p2p::identity","",1,[[],["identity",3]]],[11,"clone","locha_p2p::runtime","",10,[[],["runtime",3]]],[11,"clone","locha_p2p::msg::items","",14,[[],["content",3]]],[11,"default","","",14,[[],["content",3]]],[11,"eq","","",14,[[["content",3]]]],[11,"ne","","",14,[[["content",3]]]],[11,"fmt","locha_p2p::identity","",1,[[["formatter",3]],["result",6]]],[11,"fmt","locha_p2p::runtime::config","",6,[[["formatter",3]],["result",6]]],[11,"fmt","locha_p2p::runtime::error","",7,[[["formatter",3]],["result",6]]],[11,"fmt","locha_p2p::msg::items","",14,[[["formatter",3]],["result",6]]],[11,"fmt","locha_p2p::runtime::error","",7,[[["formatter",3]],["result",6]]],[11,"deserialize","locha_p2p::msg","",12,[[],["result",4]]],[11,"deserialize","","",13,[[],["result",4]]],[11,"serialize","","",12,[[],["result",4]]],[11,"serialize","","",13,[[],["result",4]]],[11,"new_handler","locha_p2p::p2p::behaviour","",3,[[]]],[11,"addresses_of_peer","","",3,[[["peerid",3]],[["vec",3],["multiaddr",3]]]],[11,"inject_connected","","",3,[[["peerid",3]]]],[11,"inject_disconnected","","",3,[[["peerid",3]]]],[11,"inject_connection_established","","",3,[[["connectedpoint",4],["connectionid",3],["peerid",3]]]],[11,"inject_address_change","","",3,[[["connectedpoint",4],["connectionid",3],["peerid",3]]]],[11,"inject_connection_closed","","",3,[[["connectedpoint",4],["connectionid",3],["peerid",3]]]],[11,"inject_addr_reach_failure","","",3,[[["peerid",3],["option",4],["multiaddr",3],["error",8]]]],[11,"inject_dial_failure","","",3,[[["peerid",3]]]],[11,"inject_new_listen_addr","","",3,[[["multiaddr",3]]]],[11,"inject_expired_listen_addr","","",3,[[["multiaddr",3]]]],[11,"inject_new_external_addr","","",3,[[["multiaddr",3]]]],[11,"inject_listener_error","","",3,[[["error",8],["listenerid",3]]]],[11,"inject_listener_closed","","",3,[[["error",3],["result",4],["listenerid",3]]]],[11,"inject_event","","",3,[[["connectionid",3],["peerid",3]]]],[11,"poll","","",3,[[["context",3]],[["poll",4],["networkbehaviouraction",4]]]],[11,"new_handler","locha_p2p::upnp","",11,[[]]],[11,"addresses_of_peer","","",11,[[["peerid",3]],[["multiaddr",3],["vec",3]]]],[11,"inject_connected","","",11,[[["peerid",3]]]],[11,"inject_disconnected","","",11,[[["peerid",3]]]],[11,"inject_event","","",11,[[["peerid",3],["void",4],["connectionid",3]]]],[11,"poll","","",11,[[["context",3]],[["poll",4],["networkbehaviouraction",4]]]],[11,"encode_raw","locha_p2p::msg::items","",14,[[]]],[11,"merge_field","","",14,[[["wiretype",4],["decodecontext",3]],[["decodeerror",3],["result",4]]]],[11,"encoded_len","","",14,[[]]],[11,"clear","","",14,[[]]],[11,"inject_event","locha_p2p::p2p::behaviour","",3,[[["mdnsevent",4]]]],[11,"inject_event","","",3,[[["kademliaevent",4]]]],[11,"inject_event","","",3,[[["gossipsubevent",4]]]],[11,"inject_event","","",3,[[["void",4]]]],[11,"inject_event","","",3,[[["identifyevent",4]]]],[11,"from_str_parts","locha_p2p","Parse a protocol value from the given iterator of string…",0,[[],[["protocol",4],["error",4],["result",4]]]],[11,"from_bytes","","Parse a single `Protocol` value from its byte slice…",0,[[],[["result",4],["error",4]]]],[11,"write_bytes","","Encode this protocol by writing its binary representation…",0,[[],[["result",4],["error",4]]]],[11,"acquire","","Turn this `Protocol` into one that owns its data, thus…",0,[[],["protocol",4]]],[11,"empty","","Create a new, empty multiaddress.",15,[[],["multiaddr",3]]],[11,"with_capacity","","Create a new, empty multiaddress with the given capacity.",15,[[],["multiaddr",3]]],[11,"len","","Return the length in bytes of this multiaddress.",15,[[]]],[11,"to_vec","","Return a copy of this [`Multiaddr`]\'s byte representation.",15,[[],["vec",3]]],[11,"push","","Adds an already-parsed address component to the end of…",15,[[["protocol",4]]]],[11,"pop","","Pops the last `Protocol` of this multiaddr, or `None` if…",15,[[],[["protocol",4],["option",4]]]],[11,"with","","Like [`Multiaddr::push`] but consumes `self`.",15,[[["protocol",4]],["multiaddr",3]]],[11,"iter","","Returns the components of this multiaddress.",15,[[],["iter",3]]],[11,"replace","","Replace a [`Protocol`] at some position in this `Multiaddr`.",15,[[],[["option",4],["multiaddr",3]]]],[11,"from_public_key","","Builds a `PeerId` from a public key.",16,[[["publickey",4]],["peerid",3]]],[11,"from_bytes","","Checks whether `data` is a valid `PeerId`. If so, returns…",16,[[["vec",3]],[["result",4],["vec",3],["peerid",3]]]],[11,"from_multihash","","Tries to turn a `Multihash` into a `PeerId`.",16,[[["code",4],["multihashgeneric",3]],[["result",4],["multihashgeneric",3],["peerid",3]]]],[11,"random","","Generates a random peer ID from a cryptographically secure…",16,[[],["peerid",3]]],[11,"into_bytes","","Returns a raw bytes representation of this `PeerId`.",16,[[],["vec",3]]],[11,"as_bytes","","Returns a raw bytes representation of this `PeerId`.",16,[[]]],[11,"to_base58","","Returns a base-58 encoded string of this `PeerId`.",16,[[],["string",3]]],[11,"is_public_key","","Checks whether the public key passed as parameter matches…",16,[[["publickey",4]],["option",4]]],[11,"new","locha_p2p::runtime","Constructs a new `Key` by running the given value through…",17,[[],["key",3]]],[11,"preimage","","Borrows the preimage of the key.",17,[[]]],[11,"into_preimage","","Converts the key into its preimage.",17,[[]]],[11,"distance","","Computes the distance of the keys according to the XOR…",17,[[],["distance",3]]],[11,"for_distance","","Returns the uniquely determined key with the given…",17,[[["distance",3]],["keybytes",3]]],[11,"new","","Creates a new list of addresses.",18,[[["multiaddr",3]],["addresses",3]]],[11,"first","","Gets a reference to the first address in the list.",18,[[],["multiaddr",3]]],[11,"iter","","Returns an iterator over the addresses.",18,[[]]],[11,"len","","Returns the number of addresses in the list.",18,[[]]],[11,"into_vec","","Converts the addresses into a `Vec`.",18,[[],[["vec",3],["multiaddr",3]]]],[11,"remove","","Removes the given address from the list.",18,[[["multiaddr",3]],["result",4]]],[11,"insert","","Adds a new address to the end of the list.",18,[[["multiaddr",3]]]],[11,"replace","","Replaces an old address with a new address.",18,[[["multiaddr",3]]]]],"p":[[4,"Protocol"],[3,"Identity"],[4,"BehaviourEvent"],[3,"Behaviour"],[3,"NetworkInfo"],[3,"EntryView"],[3,"RuntimeConfig"],[4,"Error"],[8,"RuntimeEvents"],[3,"RuntimeEventsLogger"],[3,"Runtime"],[3,"UpnpBehaviour"],[3,"Msg"],[3,"MessageData"],[3,"Content"],[3,"Multiaddr"],[3,"PeerId"],[3,"Key"],[3,"Addresses"]]},\
"locha_p2pd":{"doc":"","i":[[3,"EventsHandler","locha_p2pd","",null,null],[5,"deserialize_message","","",null,[[],["string",3]]],[5,"serialize_message","","",null,[[["string",3]],["vec",3]]],[5,"main","","",null,[[]]],[5,"load_identity","","",null,[[["path",3]],[["identity",3],["result",6]]]],[0,"arguments","","",null,null],[3,"Arguments","locha_p2pd::arguments","Command line arguments",null,null],[12,"listen_addr","","",0,null],[12,"peers","","",0,null],[12,"identity","","",0,null],[12,"disable_upnp","","",0,null],[12,"disable_mdns","","",0,null],[12,"dont_bootstrap","","",0,null],[11,"from_matches","","Construct the arguments from the command line matches.",0,[[["argmatches",3]],["arguments",3]]],[11,"from","locha_p2pd","",1,[[]]],[11,"into","","",1,[[]]],[11,"borrow","","",1,[[]]],[11,"borrow_mut","","",1,[[]]],[11,"try_from","","",1,[[],["result",4]]],[11,"try_into","","",1,[[],["result",4]]],[11,"type_id","","",1,[[],["typeid",3]]],[11,"vzip","","",1,[[]]],[11,"from","locha_p2pd::arguments","",0,[[]]],[11,"into","","",0,[[]]],[11,"borrow","","",0,[[]]],[11,"borrow_mut","","",0,[[]]],[11,"try_from","","",0,[[],["result",4]]],[11,"try_into","","",0,[[],["result",4]]],[11,"type_id","","",0,[[],["typeid",3]]],[11,"vzip","","",0,[[]]],[11,"fmt","","",0,[[["formatter",3]],["result",6]]],[11,"on_new_message","locha_p2pd","",1,[[["peerid",3],["vec",3]]]]],"p":[[3,"Arguments"],[3,"EventsHandler"]]},\
"miniupnpc":{"doc":"MiniUPnPThis crate contains safe bindings to the miniupnpc…","i":[[3,"Error","miniupnpc","MiniUPnP library errors.",null,null],[3,"DeviceList","","uPnP discovered devices list",null,null],[3,"DeviceListIter","","Iterator over a [`DeviceList`]",null,null],[3,"Device","","Single uPnP device.",null,null],[3,"ValidIgd","","Information about a valid IGD",null,null],[12,"urls","","",0,null],[12,"data","","",0,null],[12,"lan_address","","",0,null],[12,"connected","","",0,null],[3,"Urls","","UPnP IGD device URLs",null,null],[3,"IgdData","","IGD data",null,null],[3,"IgdService","","IGD service information",null,null],[4,"LocalPort","","",null,null],[13,"Any","","The OS assigns the source port",1,null],[13,"Same","","Uses the same destination port (1900).",1,null],[13,"Other","","Other local port.",1,null],[5,"api_version","","MiniUPnP API version",null,[[]]],[5,"version","","MiniUPnP version",null,[[]]],[5,"discover","","Discover uPnP devices.",null,[[["duration",3],["option",4],["localport",4]],[["devicelist",3],["error",3],["result",4]]]],[0,"commands","","UPnP Commands",null,null],[3,"StatusInfo","miniupnpc::commands","UPnP status information",null,null],[12,"status","","",2,null],[12,"uptime","","",2,null],[12,"lastconnerror","","",2,null],[4,"Protocol","","",null,null],[13,"Tcp","","",3,null],[13,"Udp","","",3,null],[5,"add_port_mapping","","Add port mapping",null,[[["duration",3],["option",4],["socketaddrv4",3],["protocol",4]],[["error",3],["result",4]]]],[5,"get_connection_type_info","","Get connection type information.",null,[[],[["result",4],["error",3],["string",3]]]],[5,"get_external_ip_address","","Get External IPv4 address",null,[[],[["error",3],["ipv4addr",3],["result",4]]]],[5,"get_status_info","","Get status info",null,[[],[["error",3],["result",4],["statusinfo",3]]]],[11,"iter","miniupnpc","Returns an iterator over the device list",4,[[],["devicelistiter",3]]],[11,"is_empty","","Is the device list empty?",4,[[]]],[11,"get_valid_igd","","Get valid IGD from the device list",4,[[],[["option",4],["validigd",3]]]],[11,"desc_url","","Description URL",5,[[],["cow",4]]],[11,"scope_id","","Returns the scope ID associated with this device.",5,[[]]],[11,"control_url","","Control URL of WANIPConnection",6,[[],["cow",4]]],[11,"ipcon_desc_url","","URL of the description of WANIPConnection",6,[[],["cow",4]]],[11,"control_url_cif","","Control URL of the WANCommonInterfaceConfig",6,[[],["cow",4]]],[11,"control_url_6fc","","Control URL of the WANIPv6FirewallControl",6,[[],["cow",4]]],[11,"root_desc_url","","Root description URL",6,[[],["cow",4]]],[11,"cur_elt_name","","",7,[[],["cow",4]]],[11,"url_base","","",7,[[],["cow",4]]],[11,"presentation_url","","",7,[[],["cow",4]]],[11,"level","","",7,[[]]],[11,"cif","","",7,[[],["igdservice",3]]],[11,"first","","",7,[[],["igdservice",3]]],[11,"second","","",7,[[],["igdservice",3]]],[11,"ipv6_fc","","",7,[[],["igdservice",3]]],[11,"tmp","","",7,[[],["igdservice",3]]],[11,"control_url","","",8,[[],["cow",4]]],[11,"event_sub_url","","",8,[[],["cow",4]]],[11,"scpd_url","","",8,[[],["cow",4]]],[11,"service_type","","",8,[[],["cow",4]]],[11,"from","","",9,[[]]],[11,"into","","",9,[[]]],[11,"to_owned","","",9,[[]]],[11,"clone_into","","",9,[[]]],[11,"to_string","","",9,[[],["string",3]]],[11,"borrow","","",9,[[]]],[11,"borrow_mut","","",9,[[]]],[11,"try_from","","",9,[[],["result",4]]],[11,"try_into","","",9,[[],["result",4]]],[11,"type_id","","",9,[[],["typeid",3]]],[11,"from","","",4,[[]]],[11,"into","","",4,[[]]],[11,"borrow","","",4,[[]]],[11,"borrow_mut","","",4,[[]]],[11,"try_from","","",4,[[],["result",4]]],[11,"try_into","","",4,[[],["result",4]]],[11,"type_id","","",4,[[],["typeid",3]]],[11,"from","","",10,[[]]],[11,"into","","",10,[[]]],[11,"into_iter","","",10,[[]]],[11,"borrow","","",10,[[]]],[11,"borrow_mut","","",10,[[]]],[11,"try_from","","",10,[[],["result",4]]],[11,"try_into","","",10,[[],["result",4]]],[11,"type_id","","",10,[[],["typeid",3]]],[11,"from","","",5,[[]]],[11,"into","","",5,[[]]],[11,"borrow","","",5,[[]]],[11,"borrow_mut","","",5,[[]]],[11,"try_from","","",5,[[],["result",4]]],[11,"try_into","","",5,[[],["result",4]]],[11,"type_id","","",5,[[],["typeid",3]]],[11,"from","","",0,[[]]],[11,"into","","",0,[[]]],[11,"borrow","","",0,[[]]],[11,"borrow_mut","","",0,[[]]],[11,"try_from","","",0,[[],["result",4]]],[11,"try_into","","",0,[[],["result",4]]],[11,"type_id","","",0,[[],["typeid",3]]],[11,"from","","",6,[[]]],[11,"into","","",6,[[]]],[11,"borrow","","",6,[[]]],[11,"borrow_mut","","",6,[[]]],[11,"try_from","","",6,[[],["result",4]]],[11,"try_into","","",6,[[],["result",4]]],[11,"type_id","","",6,[[],["typeid",3]]],[11,"from","","",7,[[]]],[11,"into","","",7,[[]]],[11,"borrow","","",7,[[]]],[11,"borrow_mut","","",7,[[]]],[11,"try_from","","",7,[[],["result",4]]],[11,"try_into","","",7,[[],["result",4]]],[11,"type_id","","",7,[[],["typeid",3]]],[11,"from","","",8,[[]]],[11,"into","","",8,[[]]],[11,"borrow","","",8,[[]]],[11,"borrow_mut","","",8,[[]]],[11,"try_from","","",8,[[],["result",4]]],[11,"try_into","","",8,[[],["result",4]]],[11,"type_id","","",8,[[],["typeid",3]]],[11,"from","","",1,[[]]],[11,"into","","",1,[[]]],[11,"to_owned","","",1,[[]]],[11,"clone_into","","",1,[[]]],[11,"borrow","","",1,[[]]],[11,"borrow_mut","","",1,[[]]],[11,"try_from","","",1,[[],["result",4]]],[11,"try_into","","",1,[[],["result",4]]],[11,"type_id","","",1,[[],["typeid",3]]],[11,"from","miniupnpc::commands","",2,[[]]],[11,"into","","",2,[[]]],[11,"to_owned","","",2,[[]]],[11,"clone_into","","",2,[[]]],[11,"borrow","","",2,[[]]],[11,"borrow_mut","","",2,[[]]],[11,"try_from","","",2,[[],["result",4]]],[11,"try_into","","",2,[[],["result",4]]],[11,"type_id","","",2,[[],["typeid",3]]],[11,"from","","",3,[[]]],[11,"into","","",3,[[]]],[11,"to_owned","","",3,[[]]],[11,"clone_into","","",3,[[]]],[11,"borrow","","",3,[[]]],[11,"borrow_mut","","",3,[[]]],[11,"try_from","","",3,[[],["result",4]]],[11,"try_into","","",3,[[],["result",4]]],[11,"type_id","","",3,[[],["typeid",3]]],[11,"drop","miniupnpc","",4,[[]]],[11,"drop","","",6,[[]]],[11,"next","","",10,[[],[["device",3],["option",4]]]],[11,"clone","","",9,[[],["error",3]]],[11,"clone","","",1,[[],["localport",4]]],[11,"clone","miniupnpc::commands","",3,[[],["protocol",4]]],[11,"clone","","",2,[[],["statusinfo",3]]],[11,"eq","miniupnpc","",9,[[["error",3]]]],[11,"ne","","",9,[[["error",3]]]],[11,"eq","","",1,[[["localport",4]]]],[11,"ne","","",1,[[["localport",4]]]],[11,"eq","miniupnpc::commands","",3,[[["protocol",4]]]],[11,"eq","","",2,[[["statusinfo",3]]]],[11,"ne","","",2,[[["statusinfo",3]]]],[11,"fmt","miniupnpc","",9,[[["formatter",3]],["result",6]]],[11,"fmt","","",1,[[["formatter",3]],["result",6]]],[11,"fmt","","",4,[[["formatter",3]],["result",6]]],[11,"fmt","","",10,[[["formatter",3]],["result",6]]],[11,"fmt","","",0,[[["formatter",3]],["result",6]]],[11,"fmt","","",6,[[["formatter",3]],["result",6]]],[11,"fmt","","",7,[[["formatter",3]],["result",6]]],[11,"fmt","","",8,[[["formatter",3]],["result",6]]],[11,"fmt","miniupnpc::commands","",3,[[["formatter",3]],["result",6]]],[11,"fmt","","",2,[[["formatter",3]],["result",6]]],[11,"fmt","miniupnpc","",9,[[["formatter",3]],["result",6]]],[11,"hash","miniupnpc::commands","",3,[[]]]],"p":[[3,"ValidIgd"],[4,"LocalPort"],[3,"StatusInfo"],[4,"Protocol"],[3,"DeviceList"],[3,"Device"],[3,"Urls"],[3,"IgdData"],[3,"IgdService"],[3,"Error"],[3,"DeviceListIter"]]},\
"miniupnpc_sys":{"doc":"miniupnp-sys - MiniUPnP Raw BindingsThese are the low…","i":[[3,"IGDdatas_service","miniupnpc_sys","",null,null],[12,"controlurl","","",0,null],[12,"eventsuburl","","",0,null],[12,"scpdurl","","",0,null],[12,"servicetype","","",0,null],[3,"IGDdatas","","",null,null],[12,"cureltname","","",1,null],[12,"urlbase","","",1,null],[12,"presentationurl","","",1,null],[12,"level","","",1,null],[12,"CIF","","",1,null],[12,"first","","",1,null],[12,"second","","",1,null],[12,"IPv6FC","","",1,null],[12,"tmp","","",1,null],[3,"UPNPDev","","",null,null],[12,"pNext","","",2,null],[12,"descURL","","",2,null],[12,"st","","",2,null],[12,"usn","","",2,null],[12,"scope_id","","",2,null],[12,"buffer","","",2,null],[3,"UPNParg","","",null,null],[12,"elt","","",3,null],[12,"val","","",3,null],[3,"UPNPUrls","","",null,null],[12,"controlURL","","",4,null],[12,"ipcondescURL","","",4,null],[12,"controlURL_CIF","","",4,null],[12,"controlURL_6FC","","",4,null],[12,"rootdescURL","","",4,null],[3,"PortMappingParserData","","",null,null],[5,"IGDstartelt","","",null,null],[5,"IGDendelt","","",null,null],[5,"IGDdata","","",null,null],[5,"freeUPNPDevlist","","",null,null],[5,"simpleUPnPcommand","","",null,null],[5,"upnpDiscover","","",null,null],[5,"upnpDiscoverAll","","",null,null],[5,"upnpDiscoverDevice","","",null,null],[5,"upnpDiscoverDevices","","",null,null],[5,"parserootdesc","","",null,null],[5,"UPNP_GetValidIGD","","",null,null],[5,"UPNP_GetIGDFromUrl","","",null,null],[5,"GetUPNPUrls","","",null,null],[5,"FreeUPNPUrls","","",null,null],[5,"UPNPIGD_IsConnected","","",null,null],[5,"UPNP_GetTotalBytesSent","","",null,null],[5,"UPNP_GetTotalBytesReceived","","",null,null],[5,"UPNP_GetTotalPacketsSent","","",null,null],[5,"UPNP_GetTotalPacketsReceived","","",null,null],[5,"UPNP_GetStatusInfo","","",null,null],[5,"UPNP_GetConnectionTypeInfo","","",null,null],[5,"UPNP_GetExternalIPAddress","","",null,null],[5,"UPNP_GetLinkLayerMaxBitRates","","",null,null],[5,"UPNP_AddPortMapping","","",null,null],[5,"UPNP_AddAnyPortMapping","","",null,null],[5,"UPNP_DeletePortMapping","","",null,null],[5,"UPNP_DeletePortMappingRange","","",null,null],[5,"UPNP_GetPortMappingNumberOfEntries","","",null,null],[5,"UPNP_GetSpecificPortMappingEntry","","",null,null],[5,"UPNP_GetGenericPortMappingEntry","","",null,null],[5,"UPNP_GetListOfPortMappings","","",null,null],[5,"UPNP_GetFirewallStatus","","",null,null],[5,"UPNP_GetOutboundPinholeTimeout","","",null,null],[5,"UPNP_AddPinhole","","",null,null],[5,"UPNP_UpdatePinhole","","",null,null],[5,"UPNP_DeletePinhole","","",null,null],[5,"UPNP_CheckPinholeWorking","","",null,null],[5,"UPNP_GetPinholePackets","","",null,null],[5,"strupnperror","","",null,null],[17,"MINIUPNPC_URL_MAXSIZE","","",null,null],[17,"UPNPDISCOVER_SUCCESS","","",null,null],[17,"UPNPDISCOVER_UNKNOWN_ERROR","","",null,null],[17,"UPNPDISCOVER_SOCKET_ERROR","","",null,null],[17,"UPNPDISCOVER_MEMORY_ERROR","","",null,null],[17,"MINIUPNPC_VERSION","","",null,null],[17,"MINIUPNPC_API_VERSION","","",null,null],[17,"UPNP_LOCAL_PORT_ANY","","",null,null],[17,"UPNP_LOCAL_PORT_SAME","","",null,null],[17,"UPNPCOMMAND_SUCCESS","","",null,null],[17,"UPNPCOMMAND_UNKNOWN_ERROR","","",null,null],[17,"UPNPCOMMAND_INVALID_ARGS","","",null,null],[17,"UPNPCOMMAND_HTTP_ERROR","","",null,null],[17,"UPNPCOMMAND_INVALID_RESPONSE","","",null,null],[17,"UPNPCOMMAND_MEM_ALLOC_ERROR","","",null,null],[11,"from","","",0,[[]]],[11,"into","","",0,[[]]],[11,"to_owned","","",0,[[]]],[11,"clone_into","","",0,[[]]],[11,"borrow","","",0,[[]]],[11,"borrow_mut","","",0,[[]]],[11,"try_from","","",0,[[],["result",4]]],[11,"try_into","","",0,[[],["result",4]]],[11,"type_id","","",0,[[],["typeid",3]]],[11,"from","","",1,[[]]],[11,"into","","",1,[[]]],[11,"to_owned","","",1,[[]]],[11,"clone_into","","",1,[[]]],[11,"borrow","","",1,[[]]],[11,"borrow_mut","","",1,[[]]],[11,"try_from","","",1,[[],["result",4]]],[11,"try_into","","",1,[[],["result",4]]],[11,"type_id","","",1,[[],["typeid",3]]],[11,"from","","",2,[[]]],[11,"into","","",2,[[]]],[11,"to_owned","","",2,[[]]],[11,"clone_into","","",2,[[]]],[11,"borrow","","",2,[[]]],[11,"borrow_mut","","",2,[[]]],[11,"try_from","","",2,[[],["result",4]]],[11,"try_into","","",2,[[],["result",4]]],[11,"type_id","","",2,[[],["typeid",3]]],[11,"from","","",3,[[]]],[11,"into","","",3,[[]]],[11,"to_owned","","",3,[[]]],[11,"clone_into","","",3,[[]]],[11,"borrow","","",3,[[]]],[11,"borrow_mut","","",3,[[]]],[11,"try_from","","",3,[[],["result",4]]],[11,"try_into","","",3,[[],["result",4]]],[11,"type_id","","",3,[[],["typeid",3]]],[11,"from","","",4,[[]]],[11,"into","","",4,[[]]],[11,"to_owned","","",4,[[]]],[11,"clone_into","","",4,[[]]],[11,"borrow","","",4,[[]]],[11,"borrow_mut","","",4,[[]]],[11,"try_from","","",4,[[],["result",4]]],[11,"try_into","","",4,[[],["result",4]]],[11,"type_id","","",4,[[],["typeid",3]]],[11,"from","","",5,[[]]],[11,"into","","",5,[[]]],[11,"to_owned","","",5,[[]]],[11,"clone_into","","",5,[[]]],[11,"borrow","","",5,[[]]],[11,"borrow_mut","","",5,[[]]],[11,"try_from","","",5,[[],["result",4]]],[11,"try_into","","",5,[[],["result",4]]],[11,"type_id","","",5,[[],["typeid",3]]],[11,"clone","","",0,[[],["igddatas_service",3]]],[11,"clone","","",1,[[],["igddatas",3]]],[11,"clone","","",2,[[],["upnpdev",3]]],[11,"clone","","",3,[[],["upnparg",3]]],[11,"clone","","",4,[[],["upnpurls",3]]],[11,"clone","","",5,[[],["portmappingparserdata",3]]],[11,"fmt","","",2,[[["formatter",3]],["result",6]]],[11,"fmt","","",3,[[["formatter",3]],["result",6]]],[11,"fmt","","",4,[[["formatter",3]],["result",6]]],[11,"fmt","","",5,[[["formatter",3]],["result",6]]]],"p":[[3,"IGDdatas_service"],[3,"IGDdatas"],[3,"UPNPDev"],[3,"UPNParg"],[3,"UPNPUrls"],[3,"PortMappingParserData"]]}\
}');
addSearchOptions(searchIndex);initSearch(searchIndex);