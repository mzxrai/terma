import { Container, getContainer } from "@cloudflare/containers";
import type { DurableObjectState } from "cloudflare:workers";

export class TermaContainer extends Container {
  defaultPort = 8080;
  sleepAfter = "10m";

  constructor(state: DurableObjectState, env: TermaEnv) {
    super(state, env);
    if (!env.DATABASE_URL) {
      throw new Error("DATABASE_URL secret is required for TermaContainer");
    }
    this.envVars = {
      BIND_ADDR: "0.0.0.0:8080",
      HOST: env.HOST ?? "terma.mattmay.dev",
      DATABASE_URL: env.DATABASE_URL
    };
  }
}

export default {
  async fetch(request: Request, env: WorkerEnv): Promise<Response> {
    const container = getContainer(env.TERMA_CONTAINER);
    return container.fetch(request);
  },
};

interface WorkerEnv extends TermaEnv {
  TERMA_CONTAINER: unknown;
}

interface TermaEnv {
  DATABASE_URL?: string;
  HOST?: string;
}
