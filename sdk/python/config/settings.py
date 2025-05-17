from typing import Optional, Dict, Any
import os
import json
from pathlib import Path

class Configuration:
    """
    Configuration for the IntelliRouter client.
    
    This class manages configuration settings for the IntelliRouter client,
    including API keys, base URLs, and other settings.
    
    Args:
        api_key: API key for authentication. If not provided, will be read from
            the INTELLIROUTER_API_KEY environment variable.
        base_url: Base URL for the IntelliRouter API. Defaults to http://localhost:8000.
        timeout: Timeout for API requests in seconds. Defaults to 60.
        max_retries: Maximum number of retries for failed requests. Defaults to 3.
        **kwargs: Additional configuration settings.
    """
    def __init__(
        self,
        api_key: Optional[str] = None,
        base_url: Optional[str] = None,
        timeout: Optional[int] = None,
        max_retries: Optional[int] = None,
        **kwargs,
    ):
        self.api_key = api_key or os.environ.get("INTELLIROUTER_API_KEY", "")
        self.base_url = base_url or os.environ.get("INTELLIROUTER_BASE_URL", "http://localhost:8000")
        self.timeout = timeout or int(os.environ.get("INTELLIROUTER_TIMEOUT", "60"))
        self.max_retries = max_retries or int(os.environ.get("INTELLIROUTER_MAX_RETRIES", "3"))
        
        # Store additional settings
        self._settings = kwargs
        
        # Load settings from config file if available
        self._load_from_file()
    
    def _load_from_file(self) -> None:
        """
        Load configuration from a config file.
        
        The config file can be specified using the INTELLIROUTER_CONFIG_FILE
        environment variable. If not specified, the default location is
        ~/.intellirouter/config.json.
        """
        config_file = os.environ.get(
            "INTELLIROUTER_CONFIG_FILE",
            str(Path.home() / ".intellirouter" / "config.json")
        )
        
        if os.path.exists(config_file):
            try:
                with open(config_file, "r") as f:
                    config_data = json.load(f)
                
                # Update settings from file
                if "api_key" in config_data and not self.api_key:
                    self.api_key = config_data["api_key"]
                if "base_url" in config_data and not self.base_url:
                    self.base_url = config_data["base_url"]
                if "timeout" in config_data and not self.timeout:
                    self.timeout = config_data["timeout"]
                if "max_retries" in config_data and not self.max_retries:
                    self.max_retries = config_data["max_retries"]
                
                # Update additional settings
                for key, value in config_data.items():
                    if key not in ["api_key", "base_url", "timeout", "max_retries"]:
                        self._settings[key] = value
            except Exception as e:
                # Log error but continue
                print(f"Error loading config file: {e}")
    
    def get(self, key: str, default: Any = None) -> Any:
        """
        Get a configuration setting.
        
        Args:
            key: Setting key.
            default: Default value if the key is not found.
        
        Returns:
            The setting value, or the default if not found.
        """
        return self._settings.get(key, default)
    
    def set(self, key: str, value: Any) -> None:
        """
        Set a configuration setting.
        
        Args:
            key: Setting key.
            value: Setting value.
        """
        self._settings[key] = value
    
    def to_dict(self) -> Dict[str, Any]:
        """
        Convert the configuration to a dictionary.
        
        Returns:
            Dict containing all configuration settings.
        """
        return {
            "api_key": self.api_key,
            "base_url": self.base_url,
            "timeout": self.timeout,
            "max_retries": self.max_retries,
            **self._settings,
        }