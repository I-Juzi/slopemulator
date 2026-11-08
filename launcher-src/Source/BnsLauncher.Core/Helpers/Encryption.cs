using System;
using System.Security.Cryptography;
using System.Text;

namespace BnsLauncher.Core.Helpers
{
    // Protects stored passwords with Windows DPAPI (current user scope).
    // Only the same Windows user on the same machine can decrypt them.
    public static class Encryption
    {
        private static readonly byte[] Entropy = Encoding.UTF8.GetBytes("BnsLauncher.PrivateServer.v1");

        public static string Encrypt(string plainText)
        {
            if (string.IsNullOrEmpty(plainText))
                return string.Empty;

            try
            {
                var data = ProtectedData.Protect(
                    Encoding.UTF8.GetBytes(plainText), Entropy, DataProtectionScope.CurrentUser);
                return Convert.ToBase64String(data);
            }
            catch
            {
                return string.Empty;
            }
        }

        public static string Decrypt(string encryptedText)
        {
            if (string.IsNullOrEmpty(encryptedText))
                return string.Empty;

            try
            {
                var data = ProtectedData.Unprotect(
                    Convert.FromBase64String(encryptedText), Entropy, DataProtectionScope.CurrentUser);
                return Encoding.UTF8.GetString(data);
            }
            catch
            {
                // Old (pre-DPAPI) or foreign data: the user must type the password again.
                return string.Empty;
            }
        }
    }
}
