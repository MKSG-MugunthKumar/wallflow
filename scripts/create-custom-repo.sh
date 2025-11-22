#!/bin/bash
# Create a custom DNF repository for wallflow

echo "🗄️ Creating custom DNF repository for wallflow"

REPO_DIR="repo"
REPO_NAME="wallflow-repo"

# Create repository structure
mkdir -p "$REPO_DIR/rpm"

echo "📦 Building RPM package..."
cargo generate-rpm

# Copy RPM to repo
cp target/generate-rpm/*.rpm "$REPO_DIR/rpm/"

echo "🔨 Creating repository metadata..."
cd "$REPO_DIR"

# Create repository metadata
createrepo_c rpm/

# Create repo file for users
cat > wallflow.repo << EOF
[$REPO_NAME]
name=wallflow Repository
baseurl=https://YOUR_DOMAIN/wallflow/repo/rpm/
enabled=1
gpgcheck=0
type=rpm
EOF

echo "✅ Repository created!"
echo ""
echo "📋 To use this repository:"
echo ""
echo "1. Upload the 'repo' directory to your web server"
echo "2. Users add your repository:"
echo "   sudo dnf config-manager --add-repo https://YOUR_DOMAIN/wallflow/wallflow.repo"
echo "3. Users install wallflow:"
echo "   sudo dnf install wallflow"
echo ""
echo "🔄 To update packages:"
echo "1. Build new RPM"
echo "2. Copy to repo/rpm/"
echo "3. Run: createrepo_c rpm/"
echo "4. Upload updated repo to server"

# Create upload script
cat > upload-repo.sh << 'UPLOAD_EOF'
#!/bin/bash
# Upload repository to server
# Customize this for your hosting setup

REMOTE_SERVER="your-server.com"
REMOTE_PATH="/var/www/html/wallflow/"

echo "📤 Uploading repository to server..."

# Example using rsync (adjust for your setup)
rsync -avz --progress repo/ user@$REMOTE_SERVER:$REMOTE_PATH

echo "✅ Repository uploaded!"
echo "Users can now run:"
echo "sudo dnf config-manager --add-repo https://$REMOTE_SERVER/wallflow/wallflow.repo"
echo "sudo dnf install wallflow"
UPLOAD_EOF

chmod +x upload-repo.sh

echo ""
echo "📤 Created upload-repo.sh script"
echo "   Edit the server details and run to upload your repo"