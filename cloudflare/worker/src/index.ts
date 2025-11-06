import { Container, getContainer } from "@cloudflare/containers";

export class TermaContainer extends Container {
  defaultPort = 8080;
  sleepAfter = '10m';

  constructor(state: any, env: WorkerEnv) {
    super(state, env);
    this.envVars = {
      DATABASE_URL: env.DATABASE_URL || "",
      HOST: env.HOST || "terma-worker.mzxrai.workers.dev"
    };
  }
}

export default {
  async fetch(request: Request, env: WorkerEnv): Promise<Response> {
    // Use a singleton container instance for all requests
    const container = getContainer(env.TERMA_CONTAINER, "terma-singleton");
    return container.fetch(request);
  },
};

interface WorkerEnv {
  TERMA_CONTAINER: unknown;
  DATABASE_URL?: string;
  HOST?: string;
}
