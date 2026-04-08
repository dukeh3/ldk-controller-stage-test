pipeline {
    agent { label 'docker-rust-agent-1' }

    options {
        buildDiscarder(logRotator(numToKeepStr: '10'))
        disableConcurrentBuilds()
    }

    stages {
        stage('Deploy to lp-dev') {
            steps {
                sh '''
                    set -eux
                    ssh root@lp-dev "mkdir -p /opt/ldk-controller-stage-test"
                    scp Dockerfile docker-compose.yml root@lp-dev:/opt/ldk-controller-stage-test/
                    ssh root@lp-dev "cd /opt/ldk-controller-stage-test && docker compose build --no-cache && docker compose up -d"
                '''
            }
        }
    }

    post {
        success {
            sh 'ssh root@lp-dev "docker ps --filter name=ldk-alice --filter name=ldk-bob"'
        }
    }
}
