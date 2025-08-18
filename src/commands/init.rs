use anyhow::Result;
use std::fs;
use std::path::Path;

pub fn execute(name: Option<String>, template: String) -> Result<()> {
    let project_name = if let Some(ref n) = name {
        n.clone()
    } else {
        let current_dir = std::env::current_dir()?;
        current_dir
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("my-java-project")
            .to_string()
    };

    let current_dir = std::env::current_dir()?;
    let project_dir = if name.is_some() {
        current_dir.join(&project_name)
    } else {
        current_dir.clone()
    };

    // 检查目录是否已存在
    if project_dir.exists() && project_dir != current_dir {
        return Err(anyhow::anyhow!("目录 '{}' 已存在", project_name));
    }

    // 创建项目目录
    if name.is_some() {
        fs::create_dir_all(&project_dir)?;
    }

    // 根据模板创建项目文件
    match template.as_str() {
        "maven" => create_maven_project(&project_dir, &project_name)?,
        "gradle" => create_gradle_project(&project_dir, &project_name)?,
        _ => return Err(anyhow::anyhow!("不支持的模板类型: {}", template)),
    }

    println!("✅ 项目创建成功!");
    println!("项目名称: {}", project_name);
    println!("项目类型: {}", template);
    println!("项目路径: {}", project_dir.display());

    if name.is_some() {
        println!("\n进入项目目录:");
        println!("  cd {}", project_name);
    }

    println!("\n下一步:");
    println!("  jx install    # 安装依赖");
    println!("  jx build      # 构建项目");
    println!("  jx run        # 运行项目");

    Ok(())
}

fn create_maven_project(project_dir: &Path, project_name: &str) -> Result<()> {
    // 创建目录结构
    let src_main_java = project_dir.join("src/main/java");
    let src_main_resources = project_dir.join("src/main/resources");
    let src_test_java = project_dir.join("src/test/java");
    let src_test_resources = project_dir.join("src/test/resources");

    fs::create_dir_all(&src_main_java)?;
    fs::create_dir_all(&src_main_resources)?;
    fs::create_dir_all(&src_test_java)?;
    fs::create_dir_all(&src_test_resources)?;

    // 创建pom.xml
    let pom_content = format!(
        r#"<?xml version="1.0" encoding="UTF-8"?>
<project xmlns="http://maven.apache.org/POM/4.0.0"
         xmlns:xsi="http://www.w3.org/2001/XMLSchema-instance"
         xsi:schemaLocation="http://maven.apache.org/POM/4.0.0 
         http://maven.apache.org/xsd/maven-4.0.0.xsd">
    <modelVersion>4.0.0</modelVersion>

    <groupId>com.example</groupId>
    <artifactId>{}</artifactId>
    <version>1.0.0</version>
    <packaging>jar</packaging>

    <name>{}</name>
    <description>A Java project created with jx</description>

    <properties>
        <maven.compiler.source>11</maven.compiler.source>
        <maven.compiler.target>11</maven.compiler.target>
        <project.build.sourceEncoding>UTF-8</project.build.sourceEncoding>
    </properties>

    <dependencies>
        <dependency>
            <groupId>junit</groupId>
            <artifactId>junit</artifactId>
            <version>4.13.2</version>
            <scope>test</scope>
        </dependency>
    </dependencies>

    <build>
        <plugins>
            <plugin>
                <groupId>org.apache.maven.plugins</groupId>
                <artifactId>maven-compiler-plugin</artifactId>
                <version>3.8.1</version>
                <configuration>
                    <source>11</source>
                    <target>11</target>
                </configuration>
            </plugin>
            <plugin>
                <groupId>org.apache.maven.plugins</groupId>
                <artifactId>maven-surefire-plugin</artifactId>
                <version>2.22.2</version>
            </plugin>
        </plugins>
    </build>
</project>"#,
        project_name, project_name
    );

    fs::write(project_dir.join("pom.xml"), pom_content)?;

    // 创建主类
    let main_class_content = format!(
        r#"package com.example;

/**
 * 主类 - {}
 */
public class Main {{
    public static void main(String[] args) {{
        System.out.println("Hello, {}!");
    }}
}}"#,
        project_name, project_name
    );

    let main_class_dir = src_main_java.join("com/example");
    fs::create_dir_all(&main_class_dir)?;
    fs::write(main_class_dir.join("Main.java"), main_class_content)?;

    // 创建测试类
    let test_class_content = format!(
        r#"package com.example;

import org.junit.Test;
import static org.junit.Assert.*;

/**
 * 测试类 - {}
 */
public class MainTest {{
    @Test
    public void testMain() {{
        assertTrue(true);
    }}
}}"#,
        project_name
    );

    let test_class_dir = src_test_java.join("com/example");
    fs::create_dir_all(&test_class_dir)?;
    fs::write(test_class_dir.join("MainTest.java"), test_class_content)?;

    Ok(())
}

fn create_gradle_project(project_dir: &Path, project_name: &str) -> Result<()> {
    // 创建目录结构
    let src_main_java = project_dir.join("src/main/java");
    let src_main_resources = project_dir.join("src/main/resources");
    let src_test_java = project_dir.join("src/test/java");
    let src_test_resources = project_dir.join("src/test/resources");

    fs::create_dir_all(&src_main_java)?;
    fs::create_dir_all(&src_main_resources)?;
    fs::create_dir_all(&src_test_java)?;
    fs::create_dir_all(&src_test_resources)?;

    // 创建build.gradle
    let build_gradle_content = format!(
        r#"plugins {{
    id 'java'
    id 'application'
}}

group = 'com.example'
version = '1.0.0'
sourceCompatibility = '11'

repositories {{
    mavenCentral()
}}

dependencies {{
    testImplementation 'junit:junit:4.13.2'
}}

application {{
    mainClass = 'com.example.Main'
}}

test {{
    useJUnit()
}}

jar {{
    manifest {{
        attributes 'Main-Class': 'com.example.Main'
    }}
}}"#
    );

    fs::write(project_dir.join("build.gradle"), build_gradle_content)?;

    // 创建settings.gradle
    let settings_gradle_content = format!("rootProject.name = '{}'", project_name);
    fs::write(project_dir.join("settings.gradle"), settings_gradle_content)?;

    // 创建主类
    let main_class_content = format!(
        r#"package com.example;

/**
 * 主类 - {}
 */
public class Main {{
    public static void main(String[] args) {{
        System.out.println("Hello, {}!");
    }}
}}"#,
        project_name, project_name
    );

    let main_class_dir = src_main_java.join("com/example");
    fs::create_dir_all(&main_class_dir)?;
    fs::write(main_class_dir.join("Main.java"), main_class_content)?;

    // 创建测试类
    let test_class_content = format!(
        r#"package com.example;

import org.junit.Test;
import static org.junit.Assert.*;

/**
 * 测试类 - {}
 */
public class MainTest {{
    @Test
    public void testMain() {{
        assertTrue(true);
    }}
}}"#,
        project_name
    );

    let test_class_dir = src_test_java.join("com/example");
    fs::create_dir_all(&test_class_dir)?;
    fs::write(test_class_dir.join("MainTest.java"), test_class_content)?;

    Ok(())
}
