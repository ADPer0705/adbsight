using System.ComponentModel.DataAnnotations;

namespace android_debugger.Models
{
    public class DeviceInfo
    {
        [Key]
        public int Id { get; set; }
        public string Key { get; set; } = string.Empty;
        public string Value { get; set; } = string.Empty;
        public DateTime CreatedAt { get; set; } = DateTime.Now;
    }
}
