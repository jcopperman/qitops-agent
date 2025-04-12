pipeline {
    agent {
        docker {
            image 'rust:1.77'
        }
    }
    
    environment {
        GITHUB_TOKEN = credentials('github-token')
        OPENAI_API_KEY = credentials('openai-api-key')
    }
    
    stages {
        stage('Setup') {
            steps {
                sh '''
                    git clone https://github.com/jcopperman/qitops-agent.git /tmp/qitops-agent
                    cd /tmp/qitops-agent
                    chmod +x install.sh
                    ./install.sh
                    export PATH="$HOME/.qitops/bin:$PATH"
                '''
            }
        }
        
        stage('Configure') {
            steps {
                sh '''
                    export PATH="$HOME/.qitops/bin:$PATH"
                    qitops github config --token $GITHUB_TOKEN --owner jcopperman --repo qitops-agent
                    qitops llm add --provider openai --api-key $OPENAI_API_KEY --model gpt-4
                    qitops llm default --provider openai
                '''
            }
        }
        
        stage('Test') {
            steps {
                sh 'cargo test'
            }
        }
        
        stage('PR Analysis') {
            when {
                expression { env.CHANGE_ID != null }
            }
            steps {
                sh '''
                    export PATH="$HOME/.qitops/bin:$PATH"
                    qitops run pr-analyze --pr $CHANGE_ID
                '''
            }
        }
        
        stage('Risk Assessment') {
            when {
                expression { env.CHANGE_ID != null }
            }
            steps {
                sh '''
                    export PATH="$HOME/.qitops/bin:$PATH"
                    qitops run risk --diff $CHANGE_ID --focus security,performance
                '''
            }
        }
    }
    
    post {
        always {
            archiveArtifacts artifacts: 'qitops-*.md', allowEmptyArchive: true
        }
    }
}
