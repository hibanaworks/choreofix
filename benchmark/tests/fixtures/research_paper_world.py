"""Method-neutral RPC access to the existing 3M world; no task procedure lives here."""
import asyncio
import dataclasses
import hashlib
import importlib
import json
from pathlib import Path
import sys

from benchmark.tools import load_tools


def sources():
    root = Path.cwd().resolve()
    paths = {Path(module.__file__).resolve() for module in list(sys.modules.values())
             if getattr(module, "__file__", None)
             and Path(module.__file__).resolve().is_relative_to(root / "benchmark")}
    paths.update(root / "benchmark/descriptions/3M" / name
                 for name in ("metadata.json", "tools.json"))
    return {str(path.relative_to(root)): hashlib.sha256(path.read_bytes()).hexdigest()
            for path in sorted(paths)}


async def main():
    depth, delay = int(sys.argv[1]), float(sys.argv[2])
    sim = importlib.import_module("benchmark.environments.3M.sim").MultiAuthorPaperSim()
    sim.set_scenario_depth(depth)
    sim._delay_multiplier = delay
    registry = load_tools("3M", sim=sim)

    async def request(reader, writer):
        try:
            call = json.loads(await reader.readline())
            if call == {"inspect": True}:
                response = {"complete": sim.is_complete(),
                            "violations": [dataclasses.asdict(row) for row in sim.violations],
                            "events": [dataclasses.asdict(row) for row in sim.events],
                            "source_files": sources(), "model_calls": 0, "scored_runs": 0}
            else:
                if set(call) != {"actor", "tool", "arguments"}:
                    raise ValueError("RPC requires actor, tool and arguments")
                # Any registry operation is allowed. No preferred order, source
                # shape or Hibana observation is inspected by the world.
                event_index = len(sim.events)
                result = await registry.call(call["tool"], agent_id=call["actor"],
                                             **call["arguments"])
                event = sim.events[event_index]
                if event.agent != call["actor"] or event.tool != call["tool"] or event.args != call["arguments"]:
                    raise RuntimeError("RPC reply lost its actual world event")
                response = {"result": result.to_dict(), "event_index": event_index}
            writer.write(json.dumps(response).encode() + b"\n")
            await writer.drain()
        except Exception as error:
            writer.write(json.dumps({"error": str(error)}).encode() + b"\n")
            await writer.drain()
        finally:
            writer.close()
            await writer.wait_closed()

    server = await asyncio.start_server(request, "127.0.0.1", 0)
    print(json.dumps({"address": list(server.sockets[0].getsockname()),
                      "source_files": sources(), "depth": depth, "delay_multiplier": delay}),
          flush=True)
    async with server:
        await server.serve_forever()


if __name__ == "__main__":
    asyncio.run(main())
