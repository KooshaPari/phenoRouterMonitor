# Terraform Configuration
# Multi-cloud infrastructure as code

terraform {
  required_version = ">= 1.5"
  required_providers {
    aws = {
      source  = "hashicorp/aws"
      version = "~> 5.0"
    }
    google = {
      source  = "hashicorp/google"
      version = "~> 5.0"
    }
    azurerm = {
      source  = "hashicorp/azurerm"
      version = "~> 3.0"
    }
  }
  backend "s3" {
    bucket = "terraform-state-bucket"
    key    = "infrastructure/terraform.tfstate"
    region = "us-east-1"
  }
}

# =============================================================================
# AWS RESOURCES
# =============================================================================
module "aws_ecs" {
  source = "./modules/aws-ecs"
  
  app_name        = var.app_name
  image_tag       = var.image_tag
  container_port  = var.container_port
  desired_count   = var.desired_count
  environment     = var.environment
  
  scaling_config = {
    min_capacity = 2
    max_capacity = 10
    target_cpu   = 70
  }
  
  db_config = {
    instance_class = "db.t3.medium"
    allocated_storage = 100
  }
}

# =============================================================================
# GCP RESOURCES
# =============================================================================
module "gcp_cloud_run" {
  source = "./modules/gcp-cloud-run"
  
  project_id  = var.gcp_project_id
  app_name    = var.app_name
  image       = var.gcp_image
  region      = var.gcp_region
  
  scaling_config = {
    min_instances = 1
    max_instances = 10
    concurrency   = 80
  }
}

# =============================================================================
# AZURE RESOURCES
# =============================================================================
module "azure_container Apps" {
  source = "./modules/azure-container-apps"
  
  resource_group = var.azure_resource_group
  app_name      = var.app_name
  image         = var.azure_image
  
  scaling_config = {
    min_replicas = 1
    max_replicas = 10
  }
}

# =============================================================================
# VARIABLES
# =============================================================================
variable "app_name" {
  description = "Application name"
  type        = string
  default     = "phenotype-app"
}

variable "image_tag" {
  description = "Docker image tag"
  type        = string
  default     = "latest"
}

variable "container_port" {
  description = "Container port"
  type        = number
  default     = 3000
}

variable "desired_count" {
  description = "Desired number of tasks"
  type        = number
  default     = 2
}

variable "environment" {
  description = "Environment (dev, staging, prod)"
  type        = string
  default     = "dev"
}

# GCP variables
variable "gcp_project_id" {
  type    = string
  default = ""
}

variable "gcp_image" {
  type    = string
  default = ""
}

variable "gcp_region" {
  type    = string
  default = "us-central1"
}

# Azure variables
variable "azure_resource_group" {
  type    = string
  default = ""
}

variable "azure_image" {
  type    = string
  default = ""
}
