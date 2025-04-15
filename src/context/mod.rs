use anyhow::{Result, anyhow};
use std::fs;
use std::path::{Path, PathBuf};
use std::collections::HashMap;
use walkdir::WalkDir;
use serde::{Serialize, Deserialize};
use tracing::{info, debug, warn};

/// Repository context manager
pub struct RepositoryContext {
    /// Root directory of the repository
    root_dir: PathBuf,

    /// Project information
    project_info: ProjectInfo,

    /// File structure
    file_structure: FileStructure,

    /// Cache of file contents
    file_cache: HashMap<String, String>,
}

/// Project information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectInfo {
    /// Project name
    pub name: String,

    /// Project description
    pub description: String,

    /// Project language
    pub language: String,

    /// Project version
    pub version: String,

    /// Project authors
    pub authors: Vec<String>,

    /// Project license
    pub license: String,

    /// Project repository URL
    pub repository_url: String,

    /// Project documentation
    pub documentation: String,

    /// Project coding standards
    pub coding_standards: String,
}

/// File structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileStructure {
    /// Directories in the repository
    pub directories: Vec<String>,

    /// Files in the repository
    pub files: Vec<FileInfo>,

    /// Language statistics
    pub language_stats: HashMap<String, usize>,
}

/// File information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileInfo {
    /// File path
    pub path: String,

    /// File extension
    pub extension: String,

    /// File size in bytes
    pub size: usize,

    /// Last modified timestamp
    pub last_modified: u64,
}

impl RepositoryContext {
    /// Create a new repository context
    pub fn new(root_dir: &Path) -> Result<Self> {
        info!("Creating repository context for: {}", root_dir.display());

        if !root_dir.exists() {
            warn!("Repository root directory does not exist: {}", root_dir.display());
            return Err(anyhow!("Repository root directory does not exist: {}", root_dir.display()));
        }

        if !root_dir.is_dir() {
            warn!("Repository root path is not a directory: {}", root_dir.display());
            return Err(anyhow!("Repository root path is not a directory: {}", root_dir.display()));
        }

        info!("Extracting project information...");
        let project_info = Self::extract_project_info(root_dir)?;

        info!("Scanning file structure...");
        let file_structure = Self::scan_file_structure(root_dir)?;

        info!("Repository context created successfully with {} files and {} directories",
              file_structure.files.len(), file_structure.directories.len());

        Ok(Self {
            root_dir: root_dir.to_path_buf(),
            project_info,
            file_structure,
            file_cache: HashMap::new(),
        })
    }

    /// Extract project information from repository
    fn extract_project_info(root_dir: &Path) -> Result<ProjectInfo> {
        info!("Extracting project information from {}", root_dir.display());

        // Default project info
        let mut project_info = ProjectInfo {
            name: root_dir.file_name()
                .map(|name| name.to_string_lossy().to_string())
                .unwrap_or_else(|| "Unknown".to_string()),
            description: String::new(),
            language: String::new(),
            version: String::new(),
            authors: Vec::new(),
            license: String::new(),
            repository_url: String::new(),
            documentation: String::new(),
            coding_standards: String::new(),
        };

        // Try to extract information from README.md
        let readme_path = root_dir.join("README.md");
        if readme_path.exists() {
            debug!("Found README.md, extracting information");
            let readme_content = fs::read_to_string(&readme_path)?;

            // Extract description from README
            if let Some(description) = Self::extract_description_from_readme(&readme_content) {
                project_info.description = description;
            }

            // Extract documentation from README
            project_info.documentation = readme_content;
        }

        // Try to extract information from Cargo.toml for Rust projects
        let cargo_path = root_dir.join("Cargo.toml");
        if cargo_path.exists() {
            debug!("Found Cargo.toml, extracting information");
            let cargo_content = fs::read_to_string(&cargo_path)?;

            // Extract project name
            if let Some(name) = Self::extract_value_from_toml(&cargo_content, "name") {
                project_info.name = name;
            }

            // Extract project version
            if let Some(version) = Self::extract_value_from_toml(&cargo_content, "version") {
                project_info.version = version;
            }

            // Extract project authors
            if let Some(authors) = Self::extract_array_from_toml(&cargo_content, "authors") {
                project_info.authors = authors;
            }

            // Extract project license
            if let Some(license) = Self::extract_value_from_toml(&cargo_content, "license") {
                project_info.license = license;
            }

            // Extract repository URL
            if let Some(repository) = Self::extract_value_from_toml(&cargo_content, "repository") {
                project_info.repository_url = repository;
            }

            // Set language to Rust
            project_info.language = "Rust".to_string();
        }

        // Try to extract information from package.json for JavaScript/TypeScript projects
        let package_path = root_dir.join("package.json");
        if package_path.exists() {
            debug!("Found package.json, extracting information");
            let package_content = fs::read_to_string(&package_path)?;

            // Parse JSON
            if let Ok(json) = serde_json::from_str::<serde_json::Value>(&package_content) {
                // Extract project name
                if let Some(name) = json.get("name").and_then(|v| v.as_str()) {
                    project_info.name = name.to_string();
                }

                // Extract project description
                if let Some(description) = json.get("description").and_then(|v| v.as_str()) {
                    project_info.description = description.to_string();
                }

                // Extract project version
                if let Some(version) = json.get("version").and_then(|v| v.as_str()) {
                    project_info.version = version.to_string();
                }

                // Extract project authors
                if let Some(author) = json.get("author").and_then(|v| v.as_str()) {
                    project_info.authors = vec![author.to_string()];
                } else if let Some(authors) = json.get("authors").and_then(|v| v.as_array()) {
                    project_info.authors = authors.iter()
                        .filter_map(|a| a.as_str().map(|s| s.to_string()))
                        .collect();
                }

                // Extract project license
                if let Some(license) = json.get("license").and_then(|v| v.as_str()) {
                    project_info.license = license.to_string();
                }

                // Extract repository URL
                if let Some(repository) = json.get("repository").and_then(|v| v.as_str()) {
                    project_info.repository_url = repository.to_string();
                } else if let Some(repository) = json.get("repository").and_then(|v| v.as_object()) {
                    if let Some(url) = repository.get("url").and_then(|v| v.as_str()) {
                        project_info.repository_url = url.to_string();
                    }
                }

                // Set language to JavaScript/TypeScript
                if root_dir.join("tsconfig.json").exists() {
                    project_info.language = "TypeScript".to_string();
                } else {
                    project_info.language = "JavaScript".to_string();
                }
            }
        }

        // Try to extract coding standards from .editorconfig, .eslintrc, etc.
        let mut coding_standards = Vec::new();

        // Check for .editorconfig
        let editorconfig_path = root_dir.join(".editorconfig");
        if editorconfig_path.exists() {
            let editorconfig_content = fs::read_to_string(&editorconfig_path)?;
            coding_standards.push(format!("EditorConfig:\n{}", editorconfig_content));
        }

        // Check for .eslintrc
        let eslintrc_path = root_dir.join(".eslintrc");
        if eslintrc_path.exists() {
            let eslintrc_content = fs::read_to_string(&eslintrc_path)?;
            coding_standards.push(format!("ESLint:\n{}", eslintrc_content));
        }

        // Check for rustfmt.toml
        let rustfmt_path = root_dir.join("rustfmt.toml");
        if rustfmt_path.exists() {
            let rustfmt_content = fs::read_to_string(&rustfmt_path)?;
            coding_standards.push(format!("Rustfmt:\n{}", rustfmt_content));
        }

        // Combine coding standards
        if !coding_standards.is_empty() {
            project_info.coding_standards = coding_standards.join("\n\n");
        }

        Ok(project_info)
    }

    /// Extract description from README
    fn extract_description_from_readme(readme: &str) -> Option<String> {
        // Try to find the first paragraph after the title
        let lines: Vec<&str> = readme.lines().collect();
        let mut in_description = false;
        let mut description = Vec::new();

        for line in lines {
            let trimmed = line.trim();

            // Skip title lines (starting with #)
            if trimmed.starts_with('#') {
                in_description = true;
                continue;
            }

            // If we've found the title and this is a non-empty line, it's part of the description
            if in_description && !trimmed.is_empty() {
                description.push(trimmed);
            }

            // If we've collected some description and hit an empty line, we're done
            if in_description && !description.is_empty() && trimmed.is_empty() {
                break;
            }
        }

        if description.is_empty() {
            None
        } else {
            Some(description.join(" "))
        }
    }

    /// Extract value from TOML
    fn extract_value_from_toml(toml: &str, key: &str) -> Option<String> {
        let re = regex::Regex::new(&format!(r#"{}[ \t]*=[ \t]*"([^"]+)""#, regex::escape(key))).ok()?;
        re.captures(toml).and_then(|caps| caps.get(1).map(|m| m.as_str().to_string()))
    }

    /// Extract array from TOML
    fn extract_array_from_toml(toml: &str, key: &str) -> Option<Vec<String>> {
        let re = regex::Regex::new(&format!(r#"{}[ \t]*=[ \t]*\[(.*?)\]"#, regex::escape(key))).ok()?;
        let array_str = re.captures(toml)?.get(1)?.as_str();

        let item_re = regex::Regex::new(r#""([^"]+)""#).ok()?;
        let items: Vec<String> = item_re.captures_iter(array_str)
            .filter_map(|caps| caps.get(1).map(|m| m.as_str().to_string()))
            .collect();

        if items.is_empty() {
            None
        } else {
            Some(items)
        }
    }

    /// Scan file structure
    fn scan_file_structure(root_dir: &Path) -> Result<FileStructure> {
        info!("Scanning file structure in {}", root_dir.display());

        let mut directories = Vec::new();
        let mut files = Vec::new();
        let mut language_stats: HashMap<String, usize> = HashMap::new();

        // Common directories to ignore
        let ignore_dirs = [
            ".git", "node_modules", "target", "dist", "build", "out",
            ".idea", ".vscode", ".github", "coverage", "vendor",
        ];

        // Walk the directory tree
        for entry in WalkDir::new(root_dir)
            .follow_links(false)
            .into_iter()
            .filter_entry(|e| {
                let file_name = e.file_name().to_string_lossy();
                !ignore_dirs.iter().any(|d| file_name == *d)
            })
            .filter_map(|e| e.ok())
        {
            let path = entry.path();
            let relative_path = path.strip_prefix(root_dir)
                .unwrap_or(path)
                .to_string_lossy()
                .to_string();

            if path.is_dir() {
                if !relative_path.is_empty() {
                    directories.push(relative_path);
                }
            } else if path.is_file() {
                // Get file metadata
                if let Ok(metadata) = fs::metadata(path) {
                    let extension = path.extension()
                        .map(|ext| ext.to_string_lossy().to_string())
                        .unwrap_or_default();

                    let file_info = FileInfo {
                        path: relative_path,
                        extension: extension.clone(),
                        size: metadata.len() as usize,
                        last_modified: metadata.modified()
                            .map(|time| time.duration_since(std::time::UNIX_EPOCH).unwrap().as_secs())
                            .unwrap_or(0),
                    };

                    files.push(file_info);

                    // Update language statistics
                    if !extension.is_empty() {
                        *language_stats.entry(extension).or_insert(0) += 1;
                    }
                }
            }
        }

        Ok(FileStructure {
            directories,
            files,
            language_stats,
        })
    }

    /// Get project information
    pub fn get_project_info(&self) -> &ProjectInfo {
        &self.project_info
    }

    /// Get file structure
    pub fn get_file_structure(&self) -> &FileStructure {
        &self.file_structure
    }

    /// Get file content
    pub fn get_file_content(&mut self, path: &str) -> Result<String> {
        // Check if the file is in the cache
        if let Some(content) = self.file_cache.get(path) {
            return Ok(content.clone());
        }

        // Read the file
        let file_path = self.root_dir.join(path);
        if !file_path.exists() {
            return Err(anyhow!("File does not exist: {}", path));
        }

        let content = fs::read_to_string(&file_path)?;

        // Add to cache
        self.file_cache.insert(path.to_string(), content.clone());

        Ok(content)
    }

    /// Find related files
    pub fn find_related_files(&self, path: &str, max_files: usize) -> Vec<String> {
        let file_path = Path::new(path);
        let file_name = file_path.file_name().map(|n| n.to_string_lossy().to_string());
        let file_stem = file_path.file_stem().map(|n| n.to_string_lossy().to_string());
        let parent_dir = file_path.parent().map(|p| p.to_string_lossy().to_string());

        let mut related_files = Vec::new();

        // Find files in the same directory
        if let Some(dir) = parent_dir {
            for file in &self.file_structure.files {
                let file_parent = Path::new(&file.path).parent()
                    .map(|p| p.to_string_lossy().to_string())
                    .unwrap_or_default();

                if file_parent == dir && file.path != path {
                    related_files.push(file.path.clone());
                }
            }
        }

        // Find files with similar names
        if let Some(stem) = file_stem {
            for file in &self.file_structure.files {
                let file_stem = Path::new(&file.path).file_stem()
                    .map(|s| s.to_string_lossy().to_string())
                    .unwrap_or_default();

                if file_stem.contains(&stem) && file.path != path && !related_files.contains(&file.path) {
                    related_files.push(file.path.clone());
                }
            }
        }

        // Find test files for the current file
        if let Some(name) = file_name {
            for file in &self.file_structure.files {
                let file_name = Path::new(&file.path).file_name()
                    .map(|n| n.to_string_lossy().to_string())
                    .unwrap_or_default();

                if (file_name.starts_with("test_") && file_name.contains(&name)) ||
                   (file_name.ends_with("_test.rs") && file_name.contains(&name)) ||
                   (file_name.ends_with(".test.js") && file_name.contains(&name)) ||
                   (file_name.ends_with(".spec.js") && file_name.contains(&name)) {
                    if !related_files.contains(&file.path) {
                        related_files.push(file.path.clone());
                    }
                }
            }
        }

        // Limit the number of related files
        if related_files.len() > max_files {
            related_files.truncate(max_files);
        }

        related_files
    }

    /// Find imports and dependencies for a file
    pub fn find_imports(&mut self, path: &str) -> Result<Vec<String>> {
        let content = self.get_file_content(path)?;
        let extension = Path::new(path).extension()
            .map(|ext| ext.to_string_lossy().to_string())
            .unwrap_or_default();

        let mut imports = Vec::new();

        match extension.as_str() {
            "rs" => {
                // Find Rust imports (use statements)
                let re = regex::Regex::new(r"use\s+([^;]+);").unwrap();
                for cap in re.captures_iter(&content) {
                    if let Some(import) = cap.get(1) {
                        imports.push(import.as_str().to_string());
                    }
                }

                // Find Rust external crate imports
                let re = regex::Regex::new(r"extern\s+crate\s+([^;]+);").unwrap();
                for cap in re.captures_iter(&content) {
                    if let Some(import) = cap.get(1) {
                        imports.push(format!("extern crate {}", import.as_str()));
                    }
                }
            },
            "js" | "jsx" | "ts" | "tsx" => {
                // Find JavaScript/TypeScript imports
                let re = regex::Regex::new(r#"import\s+.*?from\s+['"]([^'"]+)['"]"#).unwrap();
                for cap in re.captures_iter(&content) {
                    if let Some(import) = cap.get(1) {
                        imports.push(import.as_str().to_string());
                    }
                }

                // Find require statements
                let re = regex::Regex::new(r#"require\s*\(\s*['"]([^'"]+)['"]"#).unwrap();
                for cap in re.captures_iter(&content) {
                    if let Some(import) = cap.get(1) {
                        imports.push(import.as_str().to_string());
                    }
                }
            },
            "py" => {
                // Find Python imports
                let re = regex::Regex::new(r"(?:from|import)\s+([^\s;]+)").unwrap();
                for cap in re.captures_iter(&content) {
                    if let Some(import) = cap.get(1) {
                        imports.push(import.as_str().to_string());
                    }
                }
            },
            _ => {
                // For other file types, don't try to parse imports
            }
        }

        Ok(imports)
    }

    /// Extract function and class definitions from a file
    pub fn extract_definitions(&mut self, path: &str) -> Result<Vec<String>> {
        let content = self.get_file_content(path)?;
        let extension = Path::new(path).extension()
            .map(|ext| ext.to_string_lossy().to_string())
            .unwrap_or_default();

        let mut definitions = Vec::new();

        match extension.as_str() {
            "rs" => {
                // Find Rust function definitions
                let re = regex::Regex::new(r"(?:pub\s+)?(?:async\s+)?fn\s+([^\s\(]+)").unwrap();
                for cap in re.captures_iter(&content) {
                    if let Some(func) = cap.get(1) {
                        definitions.push(format!("fn {}", func.as_str()));
                    }
                }

                // Find Rust struct definitions
                let re = regex::Regex::new(r"(?:pub\s+)?struct\s+([^\s\{]+)").unwrap();
                for cap in re.captures_iter(&content) {
                    if let Some(struct_name) = cap.get(1) {
                        definitions.push(format!("struct {}", struct_name.as_str()));
                    }
                }

                // Find Rust enum definitions
                let re = regex::Regex::new(r"(?:pub\s+)?enum\s+([^\s\{]+)").unwrap();
                for cap in re.captures_iter(&content) {
                    if let Some(enum_name) = cap.get(1) {
                        definitions.push(format!("enum {}", enum_name.as_str()));
                    }
                }

                // Find Rust trait definitions
                let re = regex::Regex::new(r"(?:pub\s+)?trait\s+([^\s\{:]+)").unwrap();
                for cap in re.captures_iter(&content) {
                    if let Some(trait_name) = cap.get(1) {
                        definitions.push(format!("trait {}", trait_name.as_str()));
                    }
                }

                // Find Rust impl blocks
                let re = regex::Regex::new(r"impl(?:<[^>]+>)?\s+(?:[^<\s]+)(?:<[^>]+>)?\s+for\s+([^\s\{<]+)").unwrap();
                for cap in re.captures_iter(&content) {
                    if let Some(impl_name) = cap.get(1) {
                        definitions.push(format!("impl for {}", impl_name.as_str()));
                    }
                }
            },
            "js" | "jsx" | "ts" | "tsx" => {
                // Find JavaScript/TypeScript function definitions
                let re = regex::Regex::new(r"(?:function|const|let|var)\s+([^\s=\(]+)").unwrap();
                for cap in re.captures_iter(&content) {
                    if let Some(func) = cap.get(1) {
                        definitions.push(format!("function {}", func.as_str()));
                    }
                }

                // Find JavaScript/TypeScript class definitions
                let re = regex::Regex::new(r"class\s+([^\s\{]+)").unwrap();
                for cap in re.captures_iter(&content) {
                    if let Some(class_name) = cap.get(1) {
                        definitions.push(format!("class {}", class_name.as_str()));
                    }
                }

                // Find JavaScript/TypeScript interface definitions (TypeScript only)
                let re = regex::Regex::new(r"interface\s+([^\s\{]+)").unwrap();
                for cap in re.captures_iter(&content) {
                    if let Some(interface_name) = cap.get(1) {
                        definitions.push(format!("interface {}", interface_name.as_str()));
                    }
                }
            },
            "py" => {
                // Find Python function definitions
                let re = regex::Regex::new(r"def\s+([^\s\(]+)").unwrap();
                for cap in re.captures_iter(&content) {
                    if let Some(func) = cap.get(1) {
                        definitions.push(format!("def {}", func.as_str()));
                    }
                }

                // Find Python class definitions
                let re = regex::Regex::new(r"class\s+([^\s\(:]+)").unwrap();
                for cap in re.captures_iter(&content) {
                    if let Some(class_name) = cap.get(1) {
                        definitions.push(format!("class {}", class_name.as_str()));
                    }
                }
            },
            _ => {
                // For other file types, don't try to parse definitions
            }
        }

        Ok(definitions)
    }

    /// Generate repository context for prompts
    pub fn generate_context(&self, max_length: usize) -> String {
        info!("Generating repository context with max length: {}", max_length);
        let mut context = String::new();

        // Add project information
        context.push_str(&format!("Project: {}\n", self.project_info.name));

        if !self.project_info.description.is_empty() {
            context.push_str(&format!("Description: {}\n", self.project_info.description));
        }

        if !self.project_info.language.is_empty() {
            context.push_str(&format!("Language: {}\n", self.project_info.language));
        }

        if !self.project_info.version.is_empty() {
            context.push_str(&format!("Version: {}\n", self.project_info.version));
        }

        // Add repository structure summary
        context.push_str("\nRepository Structure:\n");

        // Add top-level directories
        let top_dirs: Vec<_> = self.file_structure.directories.iter()
            .filter(|d| !d.contains('/'))
            .collect();

        if !top_dirs.is_empty() {
            context.push_str("Top-level directories:\n");
            for dir in top_dirs {
                context.push_str(&format!("- {}\n", dir));
            }
        }

        // Add language statistics
        if !self.file_structure.language_stats.is_empty() {
            context.push_str("\nLanguage Statistics:\n");

            let mut stats: Vec<_> = self.file_structure.language_stats.iter().collect();
            stats.sort_by(|a, b| b.1.cmp(a.1));

            for (ext, count) in stats.iter().take(5) {
                context.push_str(&format!("- {}: {} files\n", ext, count));
            }
        }

        // Add coding standards if available
        if !self.project_info.coding_standards.is_empty() {
            context.push_str("\nCoding Standards Summary:\n");

            // Extract a summary of coding standards (first few lines)
            let standards_summary: String = self.project_info.coding_standards
                .lines()
                .take(5)
                .collect::<Vec<_>>()
                .join("\n");

            context.push_str(&standards_summary);
            context.push_str("\n");
        }

        // Truncate if too long
        if context.len() > max_length {
            info!("Context too long ({} chars), truncating to {} chars", context.len(), max_length);
            context.truncate(max_length);
            context.push_str("...");
        }

        info!("Generated repository context with {} chars", context.len());
        debug!("Repository context: {}", context);

        context
    }

    /// Generate file context for prompts
    pub fn generate_file_context(&mut self, path: &str, include_imports: bool, include_related: bool) -> Result<String> {
        info!("Generating file context for: {} (imports: {}, related: {})", path, include_imports, include_related);
        let mut context = String::new();

        // Add file information
        context.push_str(&format!("File: {}\n", path));

        // Find file in structure
        let file_info = self.file_structure.files.iter()
            .find(|f| f.path == path);

        if let Some(file_info) = file_info {
            info!("Found file info for {}", path);
            context.push_str(&format!("Extension: {}\n", file_info.extension));
            context.push_str(&format!("Size: {} bytes\n", file_info.size));
        } else {
            warn!("File not found in repository structure: {}", path);
        }

        // Add imports if requested
        if include_imports {
            match self.find_imports(path) {
                Ok(imports) => {
                    if !imports.is_empty() {
                        info!("Found {} imports for {}", imports.len(), path);
                        context.push_str("\nImports/Dependencies:\n");
                        for import in imports {
                            context.push_str(&format!("- {}\n", import));
                        }
                    } else {
                        info!("No imports found for {}", path);
                    }
                },
                Err(e) => warn!("Failed to find imports for {}: {}", path, e)
            }
        }

        // Add definitions
        match self.extract_definitions(path) {
            Ok(definitions) => {
                if !definitions.is_empty() {
                    info!("Found {} definitions for {}", definitions.len(), path);
                    context.push_str("\nDefinitions:\n");
                    for def in definitions {
                        context.push_str(&format!("- {}\n", def));
                    }
                } else {
                    info!("No definitions found for {}", path);
                }
            },
            Err(e) => warn!("Failed to extract definitions for {}: {}", path, e)
        }

        // Add related files if requested
        if include_related {
            let related_files = self.find_related_files(path, 5);
            if !related_files.is_empty() {
                info!("Found {} related files for {}", related_files.len(), path);
                context.push_str("\nRelated Files:\n");
                for related in related_files {
                    context.push_str(&format!("- {}\n", related));
                }
            } else {
                info!("No related files found for {}", path);
            }
        }

        info!("Generated file context with {} chars", context.len());
        debug!("File context: {}", context);

        Ok(context)
    }
}
