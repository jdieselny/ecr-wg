# CONTINUUM_INIT: OPERATOR SETUP

Welcome to your Continuum baseline. To instantiate your local node, you must perform the following ritual:

1. **Local Anchor Creation:** Create your local-tier directory. This directory is strictly local and will not be tracked by Git.
   ```powershell
   mkdir ../continuum-local
   ```

2. **Persistence Setup:** Inside `../continuum-local/`, create the following sub-directories:
   ```powershell
   mkdir ../continuum-local/scratchpad
   mkdir ../continuum-local/inbox
   mkdir ../continuum-local/logs
   ```

3. **Cognitive Handshake:** Run your first boot to initialize your identity.
   ```powershell
   python -m rituals.gateway
   ```

*System Note: The `continuum-local/` directory is where all your scratchpad, personal memory, and session-specific data lives. Keep it secure.*
