# Build stage
FROM node:20-alpine AS builder

WORKDIR /app

# Copy package files
COPY package*.json ./
COPY tsconfig.json ./

# Install dependencies (monorepo workspace compatible)
RUN npm install && npm cache clean --force

# Copy source code
COPY src/ ./src/

# Build TypeScript
RUN npm run build

# Production stage
FROM node:20-alpine AS production

WORKDIR /app

# Install only production dependencies
COPY package*.json ./
RUN npm install --omit=dev && npm cache clean --force

# Copy built application
COPY --from=builder /app/dist ./dist

# Copy public directory for dashboard
COPY public/ ./public/

# Create logs directory
RUN mkdir -p /app/logs

# Create non-root user
RUN addgroup -g 1001 -S nodejs
RUN adduser -S nitter -u 1001

# Set ownership
RUN chown -R nitter:nodejs /app

# Switch to non-root user
USER nitter

# Health check
HEALTHCHECK --interval=30s --timeout=10s --start-period=40s --retries=3 \
  CMD curl -f http://localhost:3001/health || exit 1

# Expose port for API service
EXPOSE 3001

# Start the service
CMD ["npm", "start"]