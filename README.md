# ssh-agenz

ssh-agenz is a manager for SSH agents (similar to `ssh-ident` or `keychain`) that:

- identifies running agents and helps you identify redundant ones
- helps you connect to a running agents
- does those things quickly and with a lightweight, cross-compiled executable

## Development

I wanted to use this as a project to try out Zig, but Zig has not made it easy. I will probably
abandon Zig for the moment and use something like Rust instead which I'm more familiar with. I
may try out Zig again later after I understand the problem domain better and Zig has more time
to bake.

I'll preserve the Zig attempt on a branch for research purposes: `zig-v1`

### Where I Got Stuck With Zig

Maybe this is better as a blog post, but eh.

My approach for identifying running agents is pretty straightforward:

1) Traverse all the files in the `/tmp` directory for agent processes
  a) 
2) Put the likely agents into an `std.ArrayList` as `Agent` structs with two fields of type `[]u8`
3) After walking every file in `/tmp`, iterate over the suspected agents, check their number of
   identities with `ssh-add -l`, and print that to the user, or an error if the command failed

Simple enough, right? 

Well, the first thing I ran into was simple enough to debug. I was doing something like

```zig
while (try walker.next()) |entry| {
    if (substring(entry.basename, "agent.") != null and substring(entry.path, "ssh-") != null) {
	const pid = agentPid(entry.basename);
	if (pid != null) {
	    var agent_pid = undefined;
	    var agent_socket_path = undefined;
	    try existing_agents.append(Agent{ .pid = &pid, .socket_path = &entry.path});
	    log.info("Found existing SSH agent with PID: {s}", .{pid.?});
	}
    }
}
```

However, this resulted in an ArrayList something like:

```
[ Agent{pid: {170, 170, 170, 170}, socket_path: {170, 170, ...}}, Agent{pid: {170...}}]
```

The strings in each `Agent`, since they were just being assigned the same pointer as the one holding
`pid` and `entry.path` which got freed with each call to `walker.next()`, would turn into each new
PID and socket path until the end of the file traversal, at which point they became empty. I think.

To fix that, I just needed to make sure to do some separate allocations and copy some memory.

```zig
var agent_pid: [10]u8 = undefined;
var agent_socket_path: [100]u8 = undefined;
std.mem.copy(u8, agent_pid[0..], pid.?[0..]);
std.mem.copy(u8, agent_socket_path[0..], entry.path[0..]);
var agent = Agent{ .pid = &agent_pid, .socket_path = &agent_socket_path };
try existing_agents.append(agent);
log.info("Found existing SSH agent with PID: {s}", .{pid.?});
```

This was an improvement, but was still wrong. The array now looked something like:

```
[ Agent{pid: "9046", socket_path: "some/path/to/agent.9046"}, Agent{pid: "9046"...}, ..]
```

The problem is similar in presentation, but at least now the values I expect are persisting after
the file traversal. I thought that making these into distinct pieces of memory by allocating them
before each `Agent` creation would work:

```zig
var agent_pid = try allocator.alloc(u8, 10);
var agent_socket_path8 = try allocator.alloc(u8, 100);
```

But no dice. At this point I'm not really sure how I can force allocation into an unused location,
which seems to be the issue, so for now, I'm hopping off the Zig train.
