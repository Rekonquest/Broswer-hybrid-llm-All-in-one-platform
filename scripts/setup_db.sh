#!/bin/bash
# Setup PostgreSQL database for Hybrid LLM Platform

set -e

# Configuration
DB_NAME="${POSTGRES_DB:-hybrid_llm}"
DB_USER="${POSTGRES_USER:-hybrid_llm_user}"
DB_PASSWORD="${POSTGRES_PASSWORD:-change_me_in_production}"
DB_HOST="${POSTGRES_HOST:-localhost}"
DB_PORT="${POSTGRES_PORT:-5432}"

echo "🚀 Setting up Hybrid LLM Platform database..."

# Check if PostgreSQL is running
if ! pg_isready -h "$DB_HOST" -p "$DB_PORT" > /dev/null 2>&1; then
    echo "❌ PostgreSQL is not running on $DB_HOST:$DB_PORT"
    echo "   Please start PostgreSQL first"
    exit 1
fi

echo "✅ PostgreSQL is running"

# Create database and user (requires superuser)
echo "📝 Creating database and user..."
psql -h "$DB_HOST" -p "$DB_PORT" -U postgres <<EOF
-- Create user if not exists
DO \$\$
BEGIN
    IF NOT EXISTS (SELECT FROM pg_user WHERE usename = '$DB_USER') THEN
        CREATE USER $DB_USER WITH PASSWORD '$DB_PASSWORD';
    END IF;
END
\$\$;

-- Create database if not exists
SELECT 'CREATE DATABASE $DB_NAME OWNER $DB_USER'
WHERE NOT EXISTS (SELECT FROM pg_database WHERE datname = '$DB_NAME')\gexec

-- Grant privileges
GRANT ALL PRIVILEGES ON DATABASE $DB_NAME TO $DB_USER;
EOF

echo "✅ Database and user created"

# Install pgvector extension (requires superuser)
echo "📦 Installing pgvector extension..."
psql -h "$DB_HOST" -p "$DB_PORT" -U postgres -d "$DB_NAME" <<EOF
CREATE EXTENSION IF NOT EXISTS vector;
EOF

echo "✅ pgvector extension installed"

# Run schema migrations
echo "🔨 Running schema migrations..."
psql -h "$DB_HOST" -p "$DB_PORT" -U "$DB_USER" -d "$DB_NAME" -f scripts/sql/001_initial_schema.sql

echo "✅ Schema migrations complete"

# Create .env file for connection
echo "📝 Creating .env file..."
cat > .env <<EOF
DATABASE_URL=postgresql://$DB_USER:$DB_PASSWORD@$DB_HOST:$DB_PORT/$DB_NAME
POSTGRES_DB=$DB_NAME
POSTGRES_USER=$DB_USER
POSTGRES_PASSWORD=$DB_PASSWORD
POSTGRES_HOST=$DB_HOST
POSTGRES_PORT=$DB_PORT
EOF

echo "✅ .env file created"

echo ""
echo "🎉 Database setup complete!"
echo ""
echo "Connection details:"
echo "  Host: $DB_HOST"
echo "  Port: $DB_PORT"
echo "  Database: $DB_NAME"
echo "  User: $DB_USER"
echo ""
echo "Connection string saved to .env file"
echo ""
echo "To connect: psql postgresql://$DB_USER:$DB_PASSWORD@$DB_HOST:$DB_PORT/$DB_NAME"
