using System.Diagnostics;
using android_debugger.Models;
using System.Text.RegularExpressions;

namespace android_debugger.Utils
{
    public class AdbTool
    {
        private readonly string _adbPath;

        public AdbTool(string adbPath = "adb")
        {
            _adbPath = adbPath;
        }

        public bool IsDeviceConnected()
        {
            string output = ExecuteAdbCommand("devices");
            return output.Contains("device") && !output.Contains("no devices");
        }        public List<DeviceInfo> GetDeviceInfo()
        {
            var deviceInfo = new List<DeviceInfo>();
            
            try
            {
                // Test command to verify shell access
                string testOutput = ExecuteAdbCommand("shell echo test");
                if (string.IsNullOrEmpty(testOutput) || !testOutput.Contains("test"))
                {
                    // Shell access issue - add diagnostic info
                    deviceInfo.Add(new DeviceInfo { Key = "Error", Value = "Unable to access device shell. Check ADB connection." });
                    return deviceInfo;
                }
                
                // Add a basic property to confirm the device is connected
                deviceInfo.Add(new DeviceInfo { Key = "Connection Status", Value = "Device Connected" });

                // Basic device information (most essential first)
                AddDeviceProperty(deviceInfo, "Model", "shell getprop ro.product.model");
                AddDeviceProperty(deviceInfo, "Brand", "shell getprop ro.product.brand");
                AddDeviceProperty(deviceInfo, "Manufacturer", "shell getprop ro.product.manufacturer");
                AddDeviceProperty(deviceInfo, "Android Version", "shell getprop ro.build.version.release");
                AddDeviceProperty(deviceInfo, "SDK Version", "shell getprop ro.build.version.sdk");
                
                // More detailed device information
                AddDeviceProperty(deviceInfo, "Device", "shell getprop ro.product.device");
                AddDeviceProperty(deviceInfo, "Product", "shell getprop ro.product.name");
                AddDeviceProperty(deviceInfo, "Build ID", "shell getprop ro.build.id");
                AddDeviceProperty(deviceInfo, "Build Fingerprint", "shell getprop ro.build.fingerprint");
                AddDeviceProperty(deviceInfo, "Kernel Version", "shell cat /proc/version");
                AddDeviceProperty(deviceInfo, "Serial Number", "shell getprop ro.serialno");
                
                // Try to get IMEI - may fail due to permissions
                try
                {
                    string imei = ExecuteAdbCommand("shell service call iphonesubinfo 1 | grep -o '[0-9a-f]\\{8\\} ' | tail -c+3 | cut -b1-6 | tr -d '\n'")?.Trim() ?? "Permission denied";
                    // Only add if we got something meaningful
                    if (!string.IsNullOrEmpty(imei) && imei.Length > 5)
                    {
                        deviceInfo.Add(new DeviceInfo { Key = "IMEI", Value = imei });
                    }
                    else
                    {
                        deviceInfo.Add(new DeviceInfo { Key = "IMEI", Value = "Requires permission" });
                    }
                }
                catch
                {
                    deviceInfo.Add(new DeviceInfo { Key = "IMEI", Value = "Unavailable" });
                }
                
                // Hardware info
                AddDeviceProperty(deviceInfo, "CPU Architecture", "shell getprop ro.product.cpu.abi");
                AddDeviceProperty(deviceInfo, "Total RAM", "shell cat /proc/meminfo | grep MemTotal");
                AddDeviceProperty(deviceInfo, "Free RAM", "shell cat /proc/meminfo | grep MemFree");
                
                // Battery and network information - more likely to have permission issues
                try 
                {
                    // Battery info
                    AddDeviceProperty(deviceInfo, "Battery Level", "shell dumpsys battery | grep level | cut -d: -f2");
                    AddDeviceProperty(deviceInfo, "Battery Status", "shell dumpsys battery | grep status | cut -d: -f2");
                    AddDeviceProperty(deviceInfo, "Battery Health", "shell dumpsys battery | grep health | head -n1 | cut -d: -f2");
                    
                    // Network info - may fail on some devices
                    AddDeviceProperty(deviceInfo, "WiFi MAC", "shell ip addr show wlan0 | grep 'link/ether ' | cut -d' ' -f6");
                    AddDeviceProperty(deviceInfo, "WiFi IP", "shell ip addr show wlan0 | grep 'inet ' | cut -d' ' -f6");
                    AddDeviceProperty(deviceInfo, "Network Operator", "shell getprop gsm.operator.alpha");
                }
                catch
                {
                    // Ignore errors for these optional properties
                }
                
                // Storage information
                try 
                {
                    AddDeviceProperty(deviceInfo, "Internal Storage", "shell df -h /data | tail -n 1");
                    AddDeviceProperty(deviceInfo, "External Storage", "shell df -h | grep /storage/ | head -n 1");
                }
                catch
                {
                    deviceInfo.Add(new DeviceInfo { Key = "Storage Info", Value = "Unavailable" });
                }
                
                // Screen information
                AddDeviceProperty(deviceInfo, "Screen Resolution", "shell wm size");
                AddDeviceProperty(deviceInfo, "Screen Density", "shell wm density");
            }
            catch (Exception ex)
            {
                // Provide error info if something went wrong
                deviceInfo.Add(new DeviceInfo { Key = "Error Occurred", Value = ex.Message });
                deviceInfo.Add(new DeviceInfo { Key = "Suggestion", Value = "Check device connection or permissions" });
            }
            
            // Ensure we return at least some information even if everything failed
            if (deviceInfo.Count == 0)
            {
                deviceInfo.Add(new DeviceInfo { Key = "Status", Value = "No device information available" });
                deviceInfo.Add(new DeviceInfo { Key = "Possible Issue", Value = "Check device connection or permissions" });
            }
            
            return deviceInfo;
        }public List<Contact> GetContacts()
        {
            // Use a single ADB command to fetch contacts instead of multiple individual queries
            var uniqueContacts = new Dictionary<string, Contact>();
            
            // Method 1: Use a direct, efficient query with ContactsContract.CommonDataKinds
            try 
            {
                // First try to get contacts with names and phone numbers
                string contactOutput = ExecuteAdbCommand(
                    "shell content query --uri content://com.android.contacts/data " +
                    "--projection display_name:data1:mimetype");
                
                string[] lines = contactOutput.Split('\n');
                
                // Process the results
                foreach (string line in lines)
                {
                    if (string.IsNullOrWhiteSpace(line)) continue;
                    
                    // Extract the display name (contact name)
                    string name = ExtractValue(line, "display_name=([^,]+)");
                    string data1 = ExtractValue(line, "data1=([^,]+)");
                    string mimetype = ExtractValue(line, "mimetype=([^,]+)");
                    
                    if (string.IsNullOrEmpty(name) || string.IsNullOrEmpty(data1)) continue;
                    
                    // Create contact if it doesn't exist
                    if (!uniqueContacts.ContainsKey(name))
                    {
                        uniqueContacts[name] = new Contact
                        {
                            Name = name,
                            PhoneNumber = "",
                            Email = ""
                        };
                    }
                    
                    // Update the appropriate field based on mimetype
                    if (mimetype.Contains("phone"))
                    {
                        uniqueContacts[name].PhoneNumber = data1;
                    }
                    else if (mimetype.Contains("email"))
                    {
                        uniqueContacts[name].Email = data1;
                    }
                }
                
                // If we have results, return them
                if (uniqueContacts.Any())
                {
                    return uniqueContacts.Values.ToList();
                }
            }
            catch {}
            
            // Method 2: Use the Contacts Provider directly (fallback)
            try
            {
                string output = ExecuteAdbCommand(
                    "shell content query --uri content://contacts/phones/people " +
                    "--projection display_name:number:primary_email");
                
                string[] lines = output.Split('\n');
                foreach (string line in lines)
                {
                    if (string.IsNullOrWhiteSpace(line)) continue;
                    
                    string name = ExtractValue(line, "display_name=([^,]+)");
                    string phone = ExtractValue(line, "number=([^,]+)");
                    string email = ExtractValue(line, "primary_email=([^,]+)");
                    
                    if (!string.IsNullOrEmpty(name))
                    {
                        uniqueContacts[name] = new Contact
                        {
                            Name = name,
                            PhoneNumber = phone,
                            Email = email
                        };
                    }
                }
                
                if (uniqueContacts.Any())
                {
                    return uniqueContacts.Values.ToList();
                }
            }
            catch {}
            
            // Method 3: Final fallback method
            try
            {
                string output = ExecuteAdbCommand("shell content query --uri content://com.android.contacts/data --projection display_name:data1:data4");
                
                string[] lines = output.Split('\n');
                foreach (string line in lines)
                {
                    if (string.IsNullOrWhiteSpace(line)) continue;

                    // Example: Row: 0 display_name=John Doe, data1=+1234567890, data4=johndoe@example.com
                    Match match = Regex.Match(line, @"display_name=(.*?), data1=(.*?), data4=(.*?)(?:, |$)");
                    if (match.Success)
                    {
                        string name = match.Groups[1].Value.Trim();
                        string data1 = match.Groups[2].Value.Trim();
                        string data4 = match.Groups[3].Value.Trim();

                        if (!string.IsNullOrEmpty(name))
                        {
                            uniqueContacts[name] = new Contact
                            {
                                Name = name,
                                PhoneNumber = data1,
                                Email = data4
                            };
                        }
                    }
                }
            }
            catch {}

            // If still empty, return at least a placeholder
            if (!uniqueContacts.Any())
            {
                uniqueContacts["No contacts found"] = new Contact
                {
                    Name = "No contacts found",
                    PhoneNumber = "N/A",
                    Email = "N/A"
                };
            }
            
            return uniqueContacts.Values.ToList();
        }public List<SMS> GetSMS()
        {
            string output = ExecuteAdbCommand("shell content query --uri content://sms --projection address:body:date:type");
            
            var smsList = new List<SMS>();
            string[] lines = output.Split('\n');
            
            foreach (string line in lines)
            {
                if (string.IsNullOrWhiteSpace(line)) continue;

                try
                {
                    // Extract separate values using individual regex patterns for more robustness
                    string address = ExtractValue(line, "address=([^,]+)");
                    string body = ExtractValue(line, "body=([^,]+(?:,[^,]+)*)(?:, (?:date|type)=|$)");  // Special handling for body that might contain commas
                    string dateStr = ExtractValue(line, "date=([^,]+)");
                    string type = ExtractValue(line, "type=([^,]+)");
                    
                    if (string.IsNullOrEmpty(address) && string.IsNullOrEmpty(body)) 
                        continue;
                    
                    // Ensure we have defaults for missing values
                    address = !string.IsNullOrEmpty(address) ? address : "(Unknown)";
                    body = !string.IsNullOrEmpty(body) ? body : "(No content)";
                    
                    // Handle date conversion safely
                    var date = DateTime.Now;
                    if (!string.IsNullOrEmpty(dateStr) && long.TryParse(dateStr, out long dateInMillis))
                    {
                        date = DateTimeOffset.FromUnixTimeMilliseconds(dateInMillis).DateTime;
                    }
                    
                    // Convert type number to readable string
                    string messageType = "Unknown";
                    switch (type)
                    {
                        case "1": messageType = "Inbox"; break;
                        case "2": messageType = "Sent"; break;
                        case "3": messageType = "Draft"; break;
                        case "4": messageType = "Outbox"; break;
                        case "5": messageType = "Failed"; break;
                        case "6": messageType = "Queued"; break;
                    }

                    smsList.Add(new SMS
                    {
                        Address = address,
                        Body = body,
                        Date = date,
                        Type = messageType
                    });
                }
                catch (Exception)
                {
                    // Skip problematic SMS entries
                    continue;
                }
            }
            
            // If no messages found with the first query, try a simpler one
            if (!smsList.Any())
            {
                try
                {
                    string simpleOutput = ExecuteAdbCommand("shell content query --uri content://sms");
                    string[] simpleLines = simpleOutput.Split('\n');
                    
                    foreach (string line in simpleLines)
                    {
                        if (string.IsNullOrWhiteSpace(line)) continue;
                        
                        try
                        {
                            // Try to extract any useful information
                            string address = ExtractValue(line, "address=([^,]+)");
                            string body = ExtractValue(line, "body=([^,]+)");
                            
                            if (!string.IsNullOrEmpty(address) || !string.IsNullOrEmpty(body))
                            {
                                smsList.Add(new SMS
                                {
                                    Address = !string.IsNullOrEmpty(address) ? address : "(Unknown)",
                                    Body = !string.IsNullOrEmpty(body) ? body : "(No content)",
                                    Date = DateTime.Now,
                                    Type = "Unknown"
                                });
                            }
                        }
                        catch
                        {
                            // Skip this entry
                        }
                    }
                }
                catch
                {
                    // Ignore errors from fallback method
                }
            }

            return smsList;
        }
        
        private string ExtractValue(string input, string pattern)
        {
            var match = Regex.Match(input, pattern);
            return match.Success ? match.Groups[1].Value.Trim() : string.Empty;
        }        public List<CallLog> GetCallLogs()
        {
            string output = ExecuteAdbCommand("shell content query --uri content://call_log/calls --projection number:type:date:duration");
            
            var callLogs = new List<CallLog>();
            string[] lines = output.Split('\n');
            
            foreach (string line in lines)
            {
                if (string.IsNullOrWhiteSpace(line)) continue;

                try
                {
                    // Extract fields individually for more robustness
                    string number = ExtractValue(line, "number=([^,]+)");
                    string type = ExtractValue(line, "type=([^,]+)");
                    string dateStr = ExtractValue(line, "date=([^,]+)");
                    string durationStr = ExtractValue(line, "duration=([^,]+)");
                    
                    if (string.IsNullOrEmpty(number)) 
                        continue;
                        
                    // Set defaults for missing values
                    number = !string.IsNullOrEmpty(number) ? number : "(Unknown)";
                    
                    // Parse date
                    var date = DateTime.Now;
                    if (!string.IsNullOrEmpty(dateStr) && long.TryParse(dateStr, out long dateInMillis))
                    {
                        date = DateTimeOffset.FromUnixTimeMilliseconds(dateInMillis).DateTime;
                    }
                    
                    // Parse duration
                    int duration = 0;
                    if (!string.IsNullOrEmpty(durationStr) && int.TryParse(durationStr, out int parsedDuration))
                    {
                        duration = parsedDuration;
                    }

                    // Convert type number to readable string
                    string callType = "Unknown";
                    switch (type)
                    {
                        case "1": callType = "Incoming"; break;
                        case "2": callType = "Outgoing"; break;
                        case "3": callType = "Missed"; break;
                        case "4": callType = "Voicemail"; break;
                        case "5": callType = "Rejected"; break;
                        case "6": callType = "Blocked"; break;
                    }

                    callLogs.Add(new CallLog
                    {
                        Number = number,
                        CallType = callType,
                        Date = date,
                        Duration = duration
                    });
                }
                catch (Exception)
                {
                    // Skip problematic call log entries
                    continue;
                }
            }
            
            // Try alternative command if no call logs found
            if (!callLogs.Any())
            {
                try
                {
                    string altOutput = ExecuteAdbCommand("shell content query --uri content://call_log/calls");
                    string[] altLines = altOutput.Split('\n');
                    
                    foreach (string line in altLines)
                    {
                        if (string.IsNullOrWhiteSpace(line)) continue;
                        
                        try
                        {
                            string number = ExtractValue(line, "number=([^,]+)");
                            if (!string.IsNullOrEmpty(number))
                            {
                                callLogs.Add(new CallLog
                                {
                                    Number = number,
                                    CallType = "Unknown",
                                    Date = DateTime.Now,
                                    Duration = 0
                                });
                            }
                        }
                        catch
                        {
                            // Skip this entry
                        }
                    }
                }
                catch
                {
                    // Ignore errors from fallback method
                }
            }

            return callLogs;
        }        public List<Account> GetAccounts()
        {
            var accounts = new List<Account>();
            
            try
            {
                // Test if we can run commands on the device
                string testOutput = ExecuteAdbCommand("shell echo test");
                if (string.IsNullOrEmpty(testOutput) || !testOutput.Contains("test"))
                {
                    // Shell access issue - add diagnostic account
                    accounts.Add(new Account { 
                        AccountType = "Error", 
                        Email = "Unable to access device shell. Check connection." 
                    });
                    return accounts;
                }
                
                // Method 1: Try the dumpsys account command
                string output = ExecuteAdbCommand("shell dumpsys account");
                
                if (!string.IsNullOrEmpty(output))
                {
                    string[] lines = output.Split('\n');
                    foreach (string line in lines)
                    {
                        if (string.IsNullOrWhiteSpace(line)) continue;
    
                        // Match both standard format and alternative formats
                        Match typeMatch = Regex.Match(line, @"Account\s+\{name=([^,]+),\s+type=([^}]+)\}");
                        if (typeMatch.Success)
                        {
                            string email = typeMatch.Groups[1].Value.Trim();
                            string accountType = typeMatch.Groups[2].Value.Trim();
    
                            accounts.Add(new Account
                            {
                                AccountType = accountType,
                                Email = email
                            });
                        }
                        
                        // Try alternative format in case the first one doesn't match
                        if (!typeMatch.Success)
                        {
                            typeMatch = Regex.Match(line, @"([^\s]+)\s+([^\s]+@[^\s]+)");
                            if (typeMatch.Success)
                            {
                                string accountType = typeMatch.Groups[1].Value.Trim();
                                string email = typeMatch.Groups[2].Value.Trim();
                                
                                if (email.Contains("@") && !accounts.Any(a => a.Email == email && a.AccountType == accountType))
                                {
                                    accounts.Add(new Account
                                    {
                                        AccountType = accountType,
                                        Email = email
                                    });
                                }
                            }
                        }
                    }
                }
                
                // Method 2: Try another approach if first method yielded no results
                if (accounts.Count == 0)
                {
                    try
                    {
                        // Try to use content provider to get accounts (may require permissions)
                        string accountsOutput = ExecuteAdbCommand("shell content query --uri content://com.google.android.gsf.gservices");
                        
                        if (!string.IsNullOrEmpty(accountsOutput))
                        {
                            string[] lines = accountsOutput.Split('\n');
                            foreach (string line in lines)
                            {
                                if (line.Contains("@"))
                                {
                                    Match emailMatch = Regex.Match(line, @"([a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,})");
                                    if (emailMatch.Success)
                                    {
                                        string email = emailMatch.Groups[1].Value.Trim();
                                        accounts.Add(new Account
                                        {
                                            AccountType = "Google",
                                            Email = email
                                        });
                                    }
                                }
                            }
                        }
                    }
                    catch
                    {
                        // Ignore errors in the fallback method
                    }
                }
                
                // Method 3: Check settings accounts.db if all else fails
                if (accounts.Count == 0)
                {
                    try 
                    {
                        string settingsOutput = ExecuteAdbCommand("shell settings list global");
                        if (!string.IsNullOrEmpty(settingsOutput))
                        {
                            string[] settings = settingsOutput.Split('\n');
                            foreach (string setting in settings)
                            {
                                if (setting.Contains("account") || setting.Contains("email"))
                                {
                                    string[] parts = setting.Split('=');
                                    if (parts.Length == 2)
                                    {
                                        accounts.Add(new Account 
                                        { 
                                            AccountType = parts[0].Trim(),
                                            Email = parts[1].Trim()
                                        });
                                    }
                                }
                            }
                        }
                    }
                    catch 
                    {
                        // Ignore errors in last resort method
                    }
                }
                
                // If we still don't have any accounts, the device may not have permission
                if (accounts.Count == 0)
                {
                    accounts.Add(new Account { 
                        AccountType = "Information",
                        Email = "No accounts found or permission denied"
                    });
                    accounts.Add(new Account {
                        AccountType = "Suggestion", 
                        Email = "Check if app has proper permissions"
                    });
                }
            }
            catch (Exception ex)
            {
                // Handle any unexpected errors
                accounts.Add(new Account { 
                    AccountType = "Error", 
                    Email = ex.Message 
                });
                accounts.Add(new Account { 
                    AccountType = "Suggestion", 
                    Email = "Check device connection and permissions" 
                });
            }

            return accounts;
        }

        private void AddDeviceProperty(List<DeviceInfo> deviceInfo, string key, string command)
        {
            try
            {
                string value = ExecuteAdbCommand(command)?.Trim() ?? "Unknown";
                deviceInfo.Add(new DeviceInfo { Key = key, Value = value });
            }
            catch
            {
                deviceInfo.Add(new DeviceInfo { Key = key, Value = "Error retrieving information" });
            }
        }        private string ExecuteAdbCommand(string command)
        {
            try
            {
                var startInfo = new ProcessStartInfo
                {
                    FileName = _adbPath,
                    Arguments = command,
                    RedirectStandardOutput = true,
                    RedirectStandardError = true,
                    UseShellExecute = false,
                    CreateNoWindow = true
                };

                using var process = Process.Start(startInfo);
                if (process != null)
                {
                    string output = process.StandardOutput.ReadToEnd();
                    string errorOutput = process.StandardError.ReadToEnd();
                    process.WaitForExit();
                    
                    // Log both output and errors for debugging
                    Debug.WriteLine($"ADB Command: {command}");
                    Debug.WriteLine($"Output: {output}");
                    
                    if (!string.IsNullOrEmpty(errorOutput))
                    {
                        Debug.WriteLine($"Error: {errorOutput}");
                        // Return the error output if there's no standard output
                        // This helps identify permission or other issues
                        if (string.IsNullOrEmpty(output))
                        {
                            return "Error: " + errorOutput;
                        }
                    }
                    
                    // Check for common permission errors in the output
                    if (output.Contains("Permission denied") || output.Contains("requires permission"))
                    {
                        Debug.WriteLine("Permission denied detected in ADB command output");
                    }
                    
                    return output;
                }
            }
            catch (Exception ex)
            {
                Debug.WriteLine($"Error executing ADB command: {ex.Message}");
                return $"Error: {ex.Message}";
            }
            return string.Empty;
        }
    }
}
