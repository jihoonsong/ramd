package config

import (
	"encoding/json"
	"flag"
	"os"
)

type ExecutionConfig struct {
}

type NetworkConfig struct {
	Namespace string   `json:"namespace"`
	Topics    []string `json:"topics"`
	MaxPeers  int      `json:"maxPeers"`
	Port      int      `json:"port"`
}

type StorageConfig struct {
}

type AppConfig struct {
	Execution ExecutionConfig `json:"executionConfig"`
	Network   NetworkConfig   `json:"networkConfig"`
	Storage   StorageConfig   `json:"storageConfig"`
}

func LoadConfig() (*AppConfig, error) {

	var configPath string
	flag.StringVar(&configPath, "config", "./config.json", "Path to configuration file")

	flag.Parse()

	cfg := &AppConfig{}

	file, err := os.Open(configPath)
	if err != nil {
		return cfg, err
	}
	defer file.Close()

	err = json.NewDecoder(file).Decode(cfg)
	if err != nil {
		return cfg, err
	}

	return cfg, nil
}
