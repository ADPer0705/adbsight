# Android Debugger

A Windows desktop application for extracting and managing data from Android devices using ADB (Android Debug Bridge).

## Features

- **SMS Messages**: Extract and save SMS messages from connected Android devices
- **Contacts**: Extract and save contacts from connected Android devices
- **Call Logs**: Extract and save call history from connected Android devices
- **Device Information**: Extract and save device specifications and settings
- **Account Information**: Extract and save account details from connected Android devices

## Prerequisites

- Windows OS
- .NET 8.0 Runtime
- ADB (Android Debug Bridge) installed and properly configured
- USB Debugging enabled on the Android device

## Setup & Configuration

1. Ensure ADB is installed on your system
2. Connect your Android device via USB
3. Enable USB Debugging in Developer Options on your Android device
4. Configure your database connection string in the application settings

## How to Use

1. Connect your Android device via USB to your computer
2. Launch the Android Debugger application
3. Navigate to the desired tab (SMS, Contacts, Call Logs, Device Info, or Accounts)
4. Click "Load from Device" to extract data from the connected device
5. Review the extracted data in the data grid view
6. Click "Save to DB" to store the data in your database for later analysis

## Database Configuration

The application uses Entity Framework Core with SQL Server. You can configure your connection string in the `AppSettings` class.

## System Requirements

- Windows OS (Windows 10 or later recommended)
- .NET 8.0 Runtime
- Minimum 4GB RAM
- 100MB disk space (additional space required for storing extracted data)

## Development

### Built With

- C# (.NET 8.0)
- Windows Forms
- Entity Framework Core
- Microsoft SQL Server

### Project Structure

- **Data/**: Database context and configuration
- **Models/**: Data models representing different types of information
- **Services/**: Business logic and data services
- **Utils/**: Utility classes including ADB tool wrapper

## License

[Specify your license information here]

## Acknowledgments

- Android Debug Bridge (ADB) tools
