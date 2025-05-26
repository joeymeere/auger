#!/usr/bin/env python3
import os
import sys
import json
import subprocess
import shutil
import glob
import re
from pathlib import Path
import logging
from typing import List, Dict, Optional

logging.basicConfig(
    level=logging.INFO,
    format='%(asctime)s - %(levelname)s - %(message)s',
    handlers=[
        logging.FileHandler("build_process.log"),
        logging.StreamHandler(sys.stdout)
    ]
)
logger = logging.getLogger(__name__)

class RustRepoBuilder:
    def __init__(self, repo_list_path: str, base_dir: str = os.path.expanduser("~")):
        """
        Initialize the Rust repository builder.
        
        Args:
            repo_list_path: Path to the JSON file containing the list of repositories
            base_dir: Base directory where operations will be performed
        """
        self.repo_list_path = repo_list_path
        self.base_dir = base_dir
        self.binaries_dir = os.path.join(base_dir, "binaries")
        self.failed_builds_path = os.path.join(base_dir, "failed.json")
        self.failed_builds = []
        
        # Create binaries directory if it doesn't exist
        os.makedirs(self.binaries_dir, exist_ok=True)
        
        # Load failed builds if file exists
        if os.path.exists(self.failed_builds_path):
            try:
                with open(self.failed_builds_path, 'r') as f:
                    self.failed_builds = json.load(f)
            except json.JSONDecodeError:
                logger.warning(f"Failed to parse {self.failed_builds_path}, creating a new file")
                self.failed_builds = []
        
    def run_command(self, command: List[str], cwd: Optional[str] = None, check: bool = True) -> subprocess.CompletedProcess:
        """Run a shell command and log the output."""
        cmd_str = " ".join(command)
        logger.info(f"Running command: {cmd_str}")
        
        try:
            result = subprocess.run(
                command,
                cwd=cwd,
                stdout=subprocess.PIPE,
                stderr=subprocess.PIPE,
                text=True,
                check=check
            )
            if result.returncode == 0:
                logger.info(f"Command succeeded: {cmd_str}")
                if result.stdout.strip():
                    logger.debug(f"Command output: {result.stdout.strip()}")
            else:
                logger.error(f"Command failed: {cmd_str}")
                logger.error(f"Error output: {result.stderr.strip()}")
            
            return result
        except subprocess.CalledProcessError as e:
            logger.error(f"Command failed with exception: {e}")
            return e
        except Exception as e:
            logger.error(f"Failed to run command: {e}")
            raise
    
    def check_command_exists(self, command: str) -> bool:
        """Check if a command exists in PATH."""
        try:
            result = self.run_command(["which", command], check=False)
            return result.returncode == 0
        except Exception:
            return False
            
    def setup_environment(self) -> bool:
        """Set up the environment with Rust, Solana CLI, and Anchor."""
        try:
            cargo_bin = os.path.expanduser("~/.cargo/bin")
            if os.path.exists(cargo_bin):
                os.environ["PATH"] = f"{cargo_bin}:{os.environ.get('PATH', '')}"
                
            solana_bin = os.path.expanduser("~/.local/share/solana/install/active_release/bin")
            if os.path.exists(solana_bin):
                os.environ["PATH"] = f"{solana_bin}:{os.environ.get('PATH', '')}"
            
            if self.check_command_exists("rustc") and self.check_command_exists("cargo"):
                logger.info("Rust is already installed")
            else:
                logger.info("Installing Rust...")
                rustup_cmd = ["curl", "--proto", "=https", "--tlsv1.2", "-sSf", "https://sh.rustup.rs", "|", "sh", "-s", "--", "-y"]
                self.run_command(["bash", "-c", " ".join(rustup_cmd)])
                os.environ["PATH"] = f"{os.path.expanduser('~/.cargo/bin')}:{os.environ.get('PATH', '')}"
            
            if self.check_command_exists("solana"):
                logger.info("Solana CLI is already installed")
            else:
                logger.info("Installing Solana CLI...")
                solana_cmd = ["sh", "-c", "$(curl -sSfL https://release.anza.xyz/v2.0.3/install)"]
                self.run_command(solana_cmd)
                os.environ["PATH"] = f"{os.path.expanduser('~/.local/share/solana/install/active_release/bin')}:{os.environ.get('PATH', '')}"
            
            if self.check_command_exists("avm"):
                logger.info("AVM is already installed")
            else:
                logger.info("Installing AVM...")
                self.run_command(["cargo", "install", "--git", "https://github.com/coral-xyz/anchor", "avm", "--force"])
            
            logger.info("Installing Anchor versions...")
            self.run_command(["avm", "install", "0.31.0"], check=False)
            self.run_command(["avm", "install", "0.29.0"], check=False)
            self.run_command(["avm", "install", "0.28.0"], check=False)
            self.run_command(["avm", "install", "0.27.0"], check=False)
            self.run_command(["avm", "install", "0.26.0"], check=False)
            
            # Set default Anchor version
            logger.info("Setting default Anchor version...")
            self.run_command(["avm", "use", "0.29.0"], check=False)
            
            rust_version = self.run_command(["rustc", "--version"])
            solana_version = self.run_command(["solana", "--version"])
            anchor_version = self.run_command(["anchor", "--version"])
            
            logger.info(f"Rust version: {rust_version.stdout.strip()}")
            logger.info(f"Solana version: {solana_version.stdout.strip()}")
            logger.info(f"Anchor version: {anchor_version.stdout.strip()}")
            
            return True
        except Exception as e:
            logger.error(f"Failed to set up environment: {e}")
            return False

    def load_repositories(self) -> List[str]:
        """Load the list of repositories from the JSON file."""
        try:
            with open(self.repo_list_path, 'r') as f:
                data = json.load(f)
            
            if isinstance(data, list):
                return data
            elif isinstance(data, dict) and "repositories" in data:
                return data["repositories"]
            else:
                logger.error(f"Unexpected JSON structure in {self.repo_list_path}")
                return []
        except (json.JSONDecodeError, FileNotFoundError) as e:
            logger.error(f"Failed to load repositories: {e}")
            return []
    
    def clone_repository(self, repo_url: str) -> Optional[str]:
        """
        Clone a repository and return the path to the cloned directory.
        
        Args:
            repo_url: URL of the repository to clone
            
        Returns:
            Path to the cloned repository or None if cloning failed
        """
        try:
            repo_name = repo_url.split("/")[-1]
            if repo_name.endswith(".git"):
                repo_name = repo_name[:-4]
            
            repo_dir = os.path.join(self.base_dir, "repos", repo_name)
            os.makedirs(os.path.dirname(repo_dir), exist_ok=True)
            
            if os.path.exists(repo_dir):
                logger.info(f"Repository directory {repo_dir} already exists, removing it")
                shutil.rmtree(repo_dir)
            
            result = self.run_command(["git", "clone", repo_url, repo_dir], check=False)
            
            if result.returncode != 0:
                logger.error(f"Failed to clone repository {repo_url}: {result.stderr}")
                return None
            
            return repo_dir
        except Exception as e:
            logger.error(f"Failed to clone repository {repo_url}: {e}")
            return None
    
    def find_so_files(self, directory: str, include_repo_so: bool = True) -> List[str]:
        """
        Find all .so files in the target directory or included with the repo.
        
        Args:
            directory: Directory to search in
            include_repo_so: Whether to include .so files that are part of the repository
            
        Returns:
            List of paths to .so files
        """
        so_files = []
        
        if include_repo_so:
            for so_file in glob.glob(f"{directory}/**/*.so", recursive=True):
                if "target/" not in so_file:
                    logger.info(f"Found pre-built .so file in repository: {so_file}")
                    so_files.append(so_file)
        
        if so_files and include_repo_so:
            return so_files
        
        target_dirs = glob.glob(f"{directory}/**/target", recursive=True)
        
        for target_dir in target_dirs:
            for so_file in glob.glob(f"{target_dir}/**/*.so", recursive=True):
                if ("debug/deps" not in so_file and 
                    "debug/build" not in so_file and 
                    "/deps/" not in so_file and 
                    not so_file.endswith("/deps")):
                    so_files.append(so_file)
        
        return so_files
    
    def parse_cargo_toml(self, cargo_toml_path: str) -> List[str]:
        """
        Parse Cargo.toml to find workspaces and packages.
        
        Args:
            cargo_toml_path: Path to Cargo.toml file
            
        Returns:
            List of directories containing packages
        """
        package_dirs = []
        
        try:
            with open(cargo_toml_path, 'r') as f:
                content = f.read()
            workspace_match = re.search(r'\[workspace\](.*?)(\[|\Z)', content, re.DOTALL)
            if workspace_match:
                workspace_section = workspace_match.group(1)
                members_match = re.search(r'members\s*=\s*\[(.*?)\]', workspace_section, re.DOTALL)
                if members_match:
                    members_str = members_match.group(1)
                    members = re.findall(r'"([^"]+)"', members_str)
                    
                    for member in members:
                        member_path = os.path.join(os.path.dirname(cargo_toml_path), member)
                        if os.path.exists(member_path):
                            package_dirs.append(member_path)
                            
            return package_dirs
        except Exception as e:
            logger.error(f"Failed to parse Cargo.toml at {cargo_toml_path}: {e}")
            return []
        
    def find_program_directories(self, repo_dir: str) -> List[str]:
        """
        Find all directories that might contain Solana programs.
        
        Args:
            repo_dir: Path to the repository directory
            
        Returns:
            List of potential program directories
        """
        program_dirs = []
        for dir_name in ["programs", "program"]:
            potential_dir = os.path.join(repo_dir, dir_name)
            if os.path.exists(potential_dir) and os.path.isdir(potential_dir):
                program_dirs.append(potential_dir)
                for subdir in os.listdir(potential_dir):
                    subdir_path = os.path.join(potential_dir, subdir)
                    if os.path.isdir(subdir_path) and os.path.exists(os.path.join(subdir_path, "Cargo.toml")):
                        program_dirs.append(subdir_path)
        cargo_toml_path = os.path.join(repo_dir, "Cargo.toml")
        if os.path.exists(cargo_toml_path):
            workspace_packages = self.parse_cargo_toml(cargo_toml_path)
            program_dirs.extend(workspace_packages)
        for root, dirs, _ in os.walk(repo_dir):
            for dir_name in dirs:
                if "program" in dir_name.lower() and dir_name not in ["programs", "program"]:
                    program_path = os.path.join(root, dir_name)
                    if os.path.exists(os.path.join(program_path, "Cargo.toml")):
                        program_dirs.append(program_path)
        
        return program_dirs
        
    def fix_cargo_lock_version(self, directory: str) -> bool:
        """
        Fix Cargo.lock file version issue by downgrading from version 4 to 3.
        
        Args:
            directory: Path to the repository directory
            
        Returns:
            True if fixed, False otherwise
        """
        try:
            for cargo_lock_path in glob.glob(f"{directory}/**/Cargo.lock", recursive=True):
                with open(cargo_lock_path, 'r') as f:
                    content = f.read()
                
                if re.search(r'version\s*=\s*4', content):
                    logger.info(f"Found Cargo.lock with version 4 at {cargo_lock_path}, downgrading to version 3")
                    
                    modified_content = re.sub(r'version\s*=\s*4', 'version = 3', content)
                    
                    with open(cargo_lock_path, 'w') as f:
                        f.write(modified_content)
                        
                    logger.info(f"Successfully downgraded Cargo.lock version at {cargo_lock_path}")
            
            return True
        except Exception as e:
            logger.error(f"Failed to fix Cargo.lock version: {e}")
            return False
    
    def detect_version_mismatch(self, error_output: str) -> tuple:
        """
        Detect version mismatch issues in build errors.
        
        Args:
            error_output: Error output from build command
            
        Returns:
            Tuple of (solana_version, anchor_version) to try, or (None, None) if no match
        """
        rustc_match = re.search(r'requires rustc ([0-9\.]+) or newer', error_output)
        solana_match = re.search(r'solana-program v([0-9\.]+)', error_output)
        
        solana_version = None
        anchor_version = None
        
        if rustc_match and solana_match:
            required_rust = rustc_match.group(1)
            solana_prog_ver = solana_match.group(1)
            
            logger.info(f"Detected version mismatch: requires rustc {required_rust} with solana-program {solana_prog_ver}")
            
            if solana_prog_ver.startswith("2."):
                solana_version = "stable" 
                anchor_version = "0.31.0"
            elif solana_prog_ver.startswith("1.17"):
                solana_version = "v1.17.0"
                anchor_version = "0.29.0"
            elif solana_prog_ver.startswith("1.16"):
                solana_version = "v1.16.13"
                anchor_version = "0.28.0"
            elif solana_prog_ver.startswith("1.15") or solana_prog_ver.startswith("1.14"):
                solana_version = "v1.14.29"
                anchor_version = "0.27.0"
            else:
                solana_version = "v1.14.16"
                anchor_version = "0.26.0"
        
        return solana_version, anchor_version
    
    def update_toolchain_versions(self, solana_version: str, anchor_version: str) -> tuple:
        """
        Update Solana CLI and Anchor to specific versions.
        
        Args:
            solana_version: Target Solana version or None
            anchor_version: Target Anchor version or None
            
        Returns:
            Tuple of (previous_solana, previous_anchor) versions
        """
        # Capture current versions
        prev_solana = None
        prev_anchor = None
        
        try:
            solana_result = self.run_command(["solana", "--version"], check=False)
            if solana_result.returncode == 0:
                version_match = re.search(r'solana-cli\s+([0-9\.]+)', solana_result.stdout)
                if version_match:
                    prev_solana = version_match.group(1)
                    logger.info(f"Current Solana version: {prev_solana}")
            
            anchor_result = self.run_command(["anchor", "--version"], check=False)
            if anchor_result.returncode == 0:
                version_match = re.search(r'anchor-cli\s+([0-9\.]+)', anchor_result.stdout)
                if version_match:
                    prev_anchor = version_match.group(1)
                    logger.info(f"Current Anchor version: {prev_anchor}")
            
            if solana_version:
                logger.info(f"Switching Solana CLI to version {solana_version}...")
                install_cmd = ["sh", "-c", f"$(curl -sSfL https://release.anza.xyz/{solana_version}/install)"]
                solana_install_result = self.run_command(install_cmd, check=False)
                
                if solana_install_result.returncode != 0:
                    logger.error(f"Failed to install Solana {solana_version}: {solana_install_result.stderr}")
                else:
                    os.environ["PATH"] = f"{os.path.expanduser('~/.local/share/solana/install/active_release/bin')}:{os.environ.get('PATH', '')}"
                    
                    new_version_result = self.run_command(["solana", "--version"], check=False)
                    if new_version_result.returncode == 0:
                        logger.info(f"Updated Solana CLI: {new_version_result.stdout.strip()}")
            
            if anchor_version:
                logger.info(f"Switching Anchor to version {anchor_version}...")
                avm_result = self.run_command(["avm", "use", anchor_version], check=False)
                
                if avm_result.returncode != 0:
                    logger.error(f"Failed to switch to Anchor {anchor_version}: {avm_result.stderr}")
                else:
                    new_version_result = self.run_command(["anchor", "--version"], check=False)
                    if new_version_result.returncode == 0:
                        logger.info(f"Updated Anchor: {new_version_result.stdout.strip()}")
            
            return prev_solana, prev_anchor
            
        except Exception as e:
            logger.error(f"Failed to update toolchain versions: {e}")
            return prev_solana, prev_anchor
    
    def restore_toolchain_versions(self, solana_version: str, anchor_version: str):
        """
        Restore previous Solana CLI and Anchor versions.
        
        Args:
            solana_version: Previous Solana version to restore or None
            anchor_version: Previous Anchor version to restore or None
        """
        try:
            if solana_version:
                logger.info(f"Restoring Solana CLI to version {solana_version}...")
                install_cmd = ["sh", "-c", f"$(curl -sSfL https://release.anza.xyz/v{solana_version}/install)"]
                solana_install_result = self.run_command(install_cmd, check=False)
                
                if solana_install_result.returncode != 0:
                    logger.error(f"Failed to restore Solana {solana_version}: {solana_install_result.stderr}")
                else:
                    os.environ["PATH"] = f"{os.path.expanduser('~/.local/share/solana/install/active_release/bin')}:{os.environ.get('PATH', '')}"
                    version_result = self.run_command(["solana", "--version"], check=False)
                    if version_result.returncode == 0:
                        logger.info(f"Restored Solana CLI: {version_result.stdout.strip()}")
            
            if anchor_version:
                logger.info(f"Restoring Anchor to version {anchor_version}...")
                avm_result = self.run_command(["avm", "use", anchor_version], check=False)
                
                if avm_result.returncode != 0:
                    logger.error(f"Failed to restore Anchor {anchor_version}: {avm_result.stderr}")
                else:
                    version_result = self.run_command(["anchor", "--version"], check=False)
                    if version_result.returncode == 0:
                        logger.info(f"Restored Anchor: {version_result.stdout.strip()}")
                        
        except Exception as e:
            logger.error(f"Failed to restore toolchain versions: {e}")
            
    def attempt_build(self, repo_dir: str, repo_url: str) -> bool:
        """
        Attempt to build a repository with current environment settings.
        
        Args:
            repo_dir: Path to the repository directory
            repo_url: URL of the repository (for logging purposes)
            
        Returns:
            True if the build was successful, False otherwise
        """
        build_success = False
        error_output = ""
    
        anchor_toml_path = os.path.join(repo_dir, "Anchor.toml")
        if os.path.exists(anchor_toml_path):
            logger.info(f"Found Anchor.toml in {repo_dir}, building with Anchor")
            result = self.run_command(["anchor", "build"], cwd=repo_dir, check=False)
            
            if result.returncode != 0:
                logger.warning(f"Anchor build failed for {repo_url}, trying cargo build-sbf")
                error_output = result.stderr
                result = self.run_command(["cargo", "build-sbf"], cwd=repo_dir, check=False)
                if result.returncode != 0:
                    error_output += "\n" + result.stderr
            else:
                build_success = True
        else:
            logger.info(f"No Anchor.toml found in {repo_dir}, trying cargo build-sbf")
            result = self.run_command(["cargo", "build-sbf"], cwd=repo_dir, check=False)
            if result.returncode == 0:
                build_success = True
            else:
                error_output = result.stderr
        
        if not build_success:
            program_dirs = self.find_program_directories(repo_dir)
            
            if program_dirs:
                logger.info(f"Build failed in root directory, found {len(program_dirs)} potential program directories")
                
                for program_dir in program_dirs:
                    logger.info(f"Trying to build in {program_dir}")
                    
                    self.fix_cargo_lock_version(program_dir)
                    
                    if os.path.exists(os.path.join(program_dir, "Anchor.toml")):
                        logger.info(f"Found Anchor.toml in {program_dir}, building with Anchor")
                        result = self.run_command(["anchor", "build"], cwd=program_dir, check=False)
                        if result.returncode == 0:
                            build_success = True
                        else:
                            error_output += "\n" + result.stderr
                    
                    if not build_success or result.returncode != 0:
                        logger.info(f"Trying cargo build-sbf in {program_dir}")
                        result = self.run_command(["cargo", "build-sbf"], cwd=program_dir, check=False)
                        if result.returncode == 0:
                            build_success = True
                        else:
                            error_output += "\n" + result.stderr
            else:
                logger.error(f"Build failed and no program directories found for {repo_url}")
        
        return build_success, error_output
        
    def build_repository(self, repo_dir: str, repo_url: str) -> bool:
        """
        Build a repository and copy the resulting binary to the binaries directory.
        
        Args:
            repo_dir: Path to the repository directory
            repo_url: URL of the repository (for logging purposes)
            
        Returns:
            True if the build was successful, False otherwise
        """
        try:
            self.fix_cargo_lock_version(repo_dir)
            
            pre_built_so_files = self.find_so_files(repo_dir, include_repo_so=True)
            if pre_built_so_files:
                logger.info(f"Found {len(pre_built_so_files)} pre-built .so files in the repository, skipping build")
                
                for so_file in pre_built_so_files:
                    dest_filename = f"{os.path.basename(repo_dir)}_{os.path.basename(so_file)}"
                    dest_path = os.path.join(self.binaries_dir, dest_filename)
                    
                    logger.info(f"Copying pre-built {so_file} to {dest_path}")
                    shutil.copy2(so_file, dest_path)
                
                return True
            build_success = False
            build_success, error_output = self.attempt_build(repo_dir, repo_url)
            if not build_success and error_output:
                solana_version, anchor_version = self.detect_version_mismatch(error_output)
                
                if solana_version or anchor_version:
                    logger.info(f"Detected version mismatch, trying with Solana={solana_version}, Anchor={anchor_version}")
                    prev_solana, prev_anchor = self.update_toolchain_versions(solana_version, anchor_version)
                    build_success, _ = self.attempt_build(repo_dir, repo_url)
                    self.restore_toolchain_versions(prev_solana, prev_anchor)
                else:
                    logger.info("Could not determine appropriate version from error output")
            
            so_files = self.find_so_files(repo_dir, include_repo_so=False)
            logger.info(f"Found {len(so_files)} .so files after build: {so_files}")
            
            if not so_files:
                logger.error(f"No .so files found for {repo_url}")
                self.failed_builds.append(repo_url)
                return False
            
            for so_file in so_files:
               
                parent_dir = os.path.basename(os.path.dirname(so_file))
                if parent_dir == "target" or parent_dir == "deploy":
                    dest_filename = f"{os.path.basename(repo_dir)}_{os.path.basename(so_file)}"
                else:
                    dest_filename = f"{os.path.basename(repo_dir)}_{parent_dir}_{os.path.basename(so_file)}"
                
                dest_path = os.path.join(self.binaries_dir, dest_filename)
                
                logger.info(f"Copying {so_file} to {dest_path}")
                shutil.copy2(so_file, dest_path)
            
            return True
        except Exception as e:
            logger.error(f"Failed to build repository {repo_url}: {e}")
            self.failed_builds.append(repo_url)
            return False
        except Exception as e:
            logger.error(f"Failed to build repository {repo_url}: {e}")
            self.failed_builds.append(repo_url)
            return False
    
    def sort_repositories_by_last_commit(self, repositories: List[str]) -> List[str]:
        """
        Sort repositories by the date of their last commit (most recent first).
        
        Args:
            repositories: List of repository URLs
            
        Returns:
            Sorted list of repository URLs
        """
        logger.info("Sorting repositories by last commit date...")
        
        temp_dir = os.path.join(self.base_dir, "temp_metadata")
        os.makedirs(temp_dir, exist_ok=True)
        
        repo_dates = {}
        
        for repo_url in repositories:
            try:
                repo_name = repo_url.split("/")[-1]
                if repo_name.endswith(".git"):
                    repo_name = repo_name[:-4]
                
                logger.info(f"Fetching last commit date for {repo_name}...")
            
                cmd = ["git", "ls-remote", repo_url, "HEAD"]
                result = self.run_command(cmd, check=False)
                
                if result.returncode != 0:
                    logger.warning(f"Could not fetch commit info for {repo_url}: {result.stderr}")
                    repo_dates[repo_url] = "1970-01-01 00:00:00"
                    continue
                
                commit_hash = result.stdout.strip().split()[0]
                
                if not commit_hash:
                    logger.warning(f"Could not determine commit hash for {repo_url}")
                    repo_dates[repo_url] = "1970-01-01 00:00:00"
                    continue
                
                sparse_checkout_dir = os.path.join(temp_dir, repo_name)
                
                if os.path.exists(sparse_checkout_dir):
                    shutil.rmtree(sparse_checkout_dir)
                
                os.makedirs(sparse_checkout_dir)
                
                init_cmd = ["git", "init"]
                self.run_command(init_cmd, cwd=sparse_checkout_dir, check=False)
                
                add_remote_cmd = ["git", "remote", "add", "origin", repo_url]
                self.run_command(add_remote_cmd, cwd=sparse_checkout_dir, check=False)
                fetch_cmd = ["git", "fetch", "--depth=1", "origin", commit_hash]
                fetch_result = self.run_command(fetch_cmd, cwd=sparse_checkout_dir, check=False)
                
                if fetch_result.returncode != 0:
                    logger.warning(f"Could not fetch commit for {repo_url}: {fetch_result.stderr}")
                    repo_dates[repo_url] = "1970-01-01 00:00:00"
                    continue
                
                date_cmd = ["git", "show", "-s", "--format=%ci", commit_hash]
                date_result = self.run_command(date_cmd, cwd=sparse_checkout_dir, check=False)
                
                if date_result.returncode == 0 and date_result.stdout.strip():
                    commit_date = date_result.stdout.strip()
                    logger.info(f"Repository {repo_name} last commit: {commit_date}")
                    repo_dates[repo_url] = commit_date
                else:
                    logger.warning(f"Could not get commit date for {repo_url}")
                    repo_dates[repo_url] = "1970-01-01 00:00:00"
                
            except Exception as e:
                logger.error(f"Error getting commit date for {repo_url}: {e}")
                repo_dates[repo_url] = "1970-01-01 00:00:00"
        
        try:
            shutil.rmtree(temp_dir)
        except Exception as e:
            logger.warning(f"Could not clean up temp directory: {e}")
        
        sorted_repos = sorted(repositories, key=lambda r: repo_dates.get(r, "1970-01-01 00:00:00"), reverse=True)
        
        logger.info(f"Sorted {len(sorted_repos)} repositories by last commit date")
        return sorted_repos
    
    def save_failed_builds(self):
        """Save the list of failed builds to a JSON file."""
        with open(self.failed_builds_path, 'w') as f:
            json.dump(self.failed_builds, f, indent=2)
        
        logger.info(f"Saved {len(self.failed_builds)} failed builds to {self.failed_builds_path}")
    
    def process_repositories(self):
        """Process all repositories in the list."""
        repositories = self.load_repositories()
        logger.info(f"Loaded {len(repositories)} repositories from {self.repo_list_path}")
        
        for i, repo_url in enumerate(repositories, 1):
            logger.info(f"Processing repository {i}/{len(repositories)}: {repo_url}")
            
            repo_dir = self.clone_repository(repo_url)
            if not repo_dir:
                logger.error(f"Failed to clone repository {repo_url}")
                self.failed_builds.append(repo_url)
                continue
            
            success = self.build_repository(repo_dir, repo_url)
            if success:
                logger.info(f"Successfully built repository {repo_url}")
            else:
                logger.error(f"Failed to build repository {repo_url}")
            
            self.save_failed_builds()
            logger.info(f"Cleaning up repository directory {repo_dir}")
            shutil.rmtree(repo_dir, ignore_errors=True)
    
    def run(self):
        """Run the entire build process."""
        logger.info("Starting Rust repository build process")
        
        if not self.setup_environment():
            logger.error("Failed to set up environment, exiting")
            return
        
        self.process_repositories()
        
        logger.info("Build process completed")
        logger.info(f"Processed repositories: {len(self.load_repositories())}")
        logger.info(f"Failed builds: {len(self.failed_builds)}")
        logger.info(f"Binaries directory: {self.binaries_dir}")
        logger.info(f"Failed builds file: {self.failed_builds_path}")


if __name__ == "__main__":
    if len(sys.argv) < 2:
        print("Usage: python build_repos.py <repo_list_json>")
        sys.exit(1)
    
    repo_list_path = sys.argv[1]
    builder = RustRepoBuilder(repo_list_path)
    builder.run()