using System.ComponentModel.DataAnnotations;

namespace android_debugger.Models
{
    public class Account
    {
        [Key]
        public int Id { get; set; }
        public string AccountType { get; set; } = string.Empty;
        public string Email { get; set; } = string.Empty;
        public DateTime CreatedAt { get; set; } = DateTime.Now;
    }
}
