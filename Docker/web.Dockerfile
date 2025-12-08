# =============================================================================
# CLMM Liquidity Provider - Web Dashboard Dockerfile
# =============================================================================
# Multi-stage build for optimized production image
#
# Build: docker build -f Docker/web.Dockerfile -t clmm-lp-web .
# Run:   docker run -p 3000:80 clmm-lp-web
# =============================================================================

# -----------------------------------------------------------------------------
# Stage 1: Build
# -----------------------------------------------------------------------------
FROM node:20-alpine AS builder

WORKDIR /app

# Copy package files
COPY web/package*.json ./

# Install dependencies (using npm install since package-lock.json is not committed)
RUN npm install --frozen-lockfile 2>/dev/null || npm install

# Copy source files
COPY web/ ./

# Build for production
RUN npm run build

# -----------------------------------------------------------------------------
# Stage 2: Runtime (nginx)
# -----------------------------------------------------------------------------
FROM nginx:alpine AS runtime

# Copy custom nginx config
COPY Docker/nginx.conf /etc/nginx/conf.d/default.conf

# Copy built assets from builder
COPY --from=builder /app/dist /usr/share/nginx/html

# Create non-root user
RUN adduser -D -s /bin/false nginx-user && \
    chown -R nginx-user:nginx-user /usr/share/nginx/html && \
    chown -R nginx-user:nginx-user /var/cache/nginx && \
    chown -R nginx-user:nginx-user /var/log/nginx && \
    touch /var/run/nginx.pid && \
    chown -R nginx-user:nginx-user /var/run/nginx.pid

# Expose port
EXPOSE 80

# Health check
HEALTHCHECK --interval=30s --timeout=10s --start-period=30s --retries=3 \
    CMD wget --no-verbose --tries=1 --spider http://localhost:80 || exit 1

# Run nginx
CMD ["nginx", "-g", "daemon off;"]
