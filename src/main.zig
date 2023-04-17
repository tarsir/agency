const std = @import("std");
const json_fmt = std.json.stringify;
const expect = std.testing.expect;
const print = std.debug.print;
const log = std.log;

const Agent = struct {
    pid: *[]u8,
    socket_path: *[]u8,
};

pub fn main() !void {
    var arena = std.heap.ArenaAllocator.init(std.heap.page_allocator);
    defer arena.deinit();

    const allocator = arena.allocator();

    var tmp_dir = try std.fs.openIterableDirAbsolute("/tmp", .{});
    var walker: std.fs.IterableDir.Walker = try tmp_dir.walk(allocator);
    defer walker.deinit();

    var existing_agents = std.ArrayList(Agent).init(allocator);
    defer existing_agents.deinit();

    while (try walker.next()) |entry| {
        if (substring(entry.basename, "agent.") != null and substring(entry.path, "ssh-") != null) {
            const pid = agentPid(entry.basename);
            if (pid != null) {
                var agent_pid: []u8 = try allocator.alloc(u8, 10);
                var agent_socket_path: []u8 = try allocator.alloc(u8, 100);
                std.mem.copy(u8, agent_pid[0..], pid.?[0..]);
                std.mem.copy(u8, agent_socket_path[0..], entry.path[0..]);
                log.debug("Address of pid: {s}", .{&agent_pid});
                var agent = Agent{ .pid = &agent_pid, .socket_path = &agent_socket_path };
                try existing_agents.append(agent);
                log.info("Found existing SSH agent with PID: {s}", .{pid.?});
            }
        }
    }

    var env_map: std.process.EnvMap = std.process.EnvMap.init(allocator);
    defer env_map.deinit();

    if (existing_agents.items.len > 0) {
        log.info("You have {any} existing SSH agents.", .{existing_agents.items.len});
        const command = .{ "ssh-add", "-l" };
        for (existing_agents.items) |agent| {
            log.debug("{s}", .{agent.pid.*});
            // try env_map.put("SSH_AUTH_SOCK", &agent.socket_path);
            const result = try std.ChildProcess.exec(.{ .allocator = allocator, .argv = &command, .env_map = &env_map });
            log.debug("{s}", .{result.stderr});
            log.debug("{s}", .{agent.socket_path});
            if (result.stdout.len == 0) {
                log.err("Error connecting to agent {s}", .{agent.pid});
            } else {
                log.info("{s}", .{result.stdout});
            }
        }
    }
}

fn agentPid(agent_path: []const u8) ?[]const u8 {
    if (std.mem.indexOfPos(u8, agent_path, 0, ".")) |idx| {
        return agent_path[idx + 1 ..];
    }
    return null;
}

test "agentPid" {
    const agent_path = "agent.1234";
    try expect(std.mem.eql(u8, agentPid(agent_path).?, "1234"));
}

pub fn substring(base: []const u8, search: []const u8) ?usize {
    return std.mem.indexOfPos(u8, base, 0, search);
}

test "substring" {
    const base_string = "Call of Cthulhu by HP Lovecraft. こんにちは。";

    // test unicode search
    var search_string = "は"[0..];
    try expect(substring(base_string, search_string) != null);

    // test searching the last character
    search_string = "。";
    try expect(substring(base_string, search_string) != null);

    // test unicode multi-char search strings
    var search_string2 = "こんにちは";
    try expect(substring(base_string, search_string2) != null);

    // test multi-char search strings
    var search_string3 = "Call of Cthulhu";
    try expect(substring(base_string, search_string3) != null);
}
