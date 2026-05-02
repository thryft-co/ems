import React, { useState } from "react";
import { useAuth } from "@/features/auth/context/AuthContext";
import { RegisterRequest } from "@/features/auth/types/auth";
import { Button } from "@/shared/ui/button";
import { Input } from "@/shared/ui/input";
import { Label } from "@/shared/ui/label";

interface RegisterProps {
  onSwitchToLogin: () => void;
}

const Register: React.FC<RegisterProps> = ({ onSwitchToLogin }) => {
  const { register, isLoading } = useAuth();
  const [formData, setFormData] = useState<RegisterRequest>({
    email: "",
    first_name: "",
    last_name: "",
    password: "",
  });
  const [confirmPassword, setConfirmPassword] = useState("");
  const [errors, setErrors] = useState<Record<string, string>>({});
  const [submitError, setSubmitError] = useState<string>("");

  const handleChange = (e: React.ChangeEvent<HTMLInputElement>) => {
    const { name, value } = e.target;
    if (name === "confirmPassword") {
      setConfirmPassword(value);
    } else {
      setFormData((prev) => ({ ...prev, [name]: value }));
    }
    if (errors[name]) {
      setErrors((prev) => ({ ...prev, [name]: "" }));
    }
  };

  const validateForm = (): boolean => {
    const newErrors: Record<string, string> = {};
    if (!formData.email) newErrors.email = "Email is required";
    else if (!/\S+@\S+\.\S+/.test(formData.email)) newErrors.email = "Email is invalid";
    if (!formData.first_name) newErrors.first_name = "First name is required";
    else if (formData.first_name.length > 50) newErrors.first_name = "First name must be 50 characters or less";
    if (!formData.last_name) newErrors.last_name = "Last name is required";
    else if (formData.last_name.length > 50) newErrors.last_name = "Last name must be 50 characters or less";
    if (!formData.password) newErrors.password = "Password is required";
    else if (formData.password.length < 8) newErrors.password = "Password must be at least 8 characters";
    else if (formData.password.length > 128) newErrors.password = "Password must be 128 characters or less";
    if (!confirmPassword) newErrors.confirmPassword = "Please confirm your password";
    else if (confirmPassword !== formData.password) newErrors.confirmPassword = "Passwords do not match";
    setErrors(newErrors);
    return Object.keys(newErrors).length === 0;
  };

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    setSubmitError("");
    if (!validateForm()) return;
    try {
      await register(formData);
    } catch (error) {
      setSubmitError(error instanceof Error ? error.message : "Registration failed");
    }
  };

  return (
    <div className="min-h-screen flex items-center justify-center bg-background px-4 sm:px-6 py-8">
      <div className="w-full max-w-sm">
        <div className="text-center mb-8">
          <div className="mx-auto w-14 h-14 rounded-[16px] bg-primary flex items-center justify-center shadow-soft-md mb-5">
            <span className="text-white font-bold text-2xl">E</span>
          </div>
          <h1 className="text-[28px] font-bold tracking-tight text-foreground mb-1">Create Account</h1>
          <p className="text-[15px] text-muted-foreground">Join the Enterprise Management Suite</p>
        </div>

        <div className="bg-card rounded-2xl border-[0.5px] border-border/60 shadow-soft p-5 sm:p-6">
          <form onSubmit={handleSubmit} className="space-y-4">
            <div className="grid grid-cols-1 sm:grid-cols-2 gap-4">
              <div>
                <Label htmlFor="first_name">First Name</Label>
                <Input id="first_name" name="first_name" type="text" value={formData.first_name} onChange={handleChange} placeholder="First name" className={errors.first_name ? "ring-2 ring-destructive/30" : ""} disabled={isLoading} />
                {errors.first_name && <p className="mt-1.5 text-[13px] text-destructive">{errors.first_name}</p>}
              </div>
              <div>
                <Label htmlFor="last_name">Last Name</Label>
                <Input id="last_name" name="last_name" type="text" value={formData.last_name} onChange={handleChange} placeholder="Last name" className={errors.last_name ? "ring-2 ring-destructive/30" : ""} disabled={isLoading} />
                {errors.last_name && <p className="mt-1.5 text-[13px] text-destructive">{errors.last_name}</p>}
              </div>
            </div>

            <div>
              <Label htmlFor="email">Email Address</Label>
              <Input id="email" name="email" type="email" value={formData.email} onChange={handleChange} placeholder="you@example.com" className={errors.email ? "ring-2 ring-destructive/30" : ""} disabled={isLoading} />
              {errors.email && <p className="mt-1.5 text-[13px] text-destructive">{errors.email}</p>}
            </div>

            <div>
              <Label htmlFor="password">Password</Label>
              <Input id="password" name="password" type="password" value={formData.password} onChange={handleChange} placeholder="Create a password" className={errors.password ? "ring-2 ring-destructive/30" : ""} disabled={isLoading} />
              {errors.password && <p className="mt-1.5 text-[13px] text-destructive">{errors.password}</p>}
            </div>

            <div>
              <Label htmlFor="confirmPassword">Confirm Password</Label>
              <Input id="confirmPassword" name="confirmPassword" type="password" value={confirmPassword} onChange={handleChange} placeholder="Confirm your password" className={errors.confirmPassword ? "ring-2 ring-destructive/30" : ""} disabled={isLoading} />
              {errors.confirmPassword && <p className="mt-1.5 text-[13px] text-destructive">{errors.confirmPassword}</p>}
            </div>

            {submitError && <p className="text-[13px] text-destructive text-center py-1">{submitError}</p>}

            <Button type="submit" className="w-full" size="lg" disabled={isLoading}>
              {isLoading ? "Creating Account..." : "Create Account"}
            </Button>
          </form>
        </div>

        <div className="mt-6 text-center">
          <p className="text-[14px] text-muted-foreground">
            Already have an account?{" "}
            <button type="button" onClick={onSwitchToLogin} className="font-semibold text-primary hover:text-primary/80 transition-colors" disabled={isLoading}>Sign in</button>
          </p>
        </div>
      </div>
    </div>
  );
};

export default Register;
