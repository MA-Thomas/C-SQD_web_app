import type { NextConfig } from "next";

const nextConfig: NextConfig = {
  // Overridable so sandboxed/CI builds can write outside the source tree.
  distDir: process.env.NEXT_DIST_DIR ?? ".next",
};

export default nextConfig;
