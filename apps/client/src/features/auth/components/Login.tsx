import React, { useState } from "react";
import { useAuth } from "@/features/auth/context/AuthContext";
import { LoginRequest } from "@/features/auth/types/auth";
import { Button } from "@/shared/ui/button";
import { Input } from "@/shared/ui/input";
import { Label } from "@/shared/ui/label";

interface LoginProps {
  onSwitchToRegister: () => void;
}

const Login: React.FC<LoginProps> = ({ onSwitchToRegister }) => {
  const { login, isLoading } = useAuth();
  const [formData, setFormData] = useState<LoginRequest>({
    email: "",
    password: "",
  });
  const [errors, setErrors] = useState<Record<string, string>>({});
  const [submitError, setSubmitError] = useState<string>("");

  // Handle input changes
  const handleChange = (e: React.ChangeEvent<HTMLInputElement>) => {
    const { name, value } = e.target;
    setFormData((prev) => ({
      ...prev,
      [name]: value,
    }));

    // Clear field error when user starts typing
    if (errors[name]) {
      setErrors((prev) => ({
        ...prev,
        [name]: "",
      }));
    }
  };

  // Validate form
  const validateForm = (): boolean => {
    const newErrors: Record<string, string> = {};

    if (!formData.email) {
      newErrors.email = "Email is required";
    } else if (!/\S+@\S+\.\S+/.test(formData.email)) {
      newErrors.email = "Email is invalid";
    }

    if (!formData.password) {
      newErrors.password = "Password is required";
    }

    setErrors(newErrors);
    return Object.keys(newErrors).length === 0;
  };

  // Handle form submission
  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    setSubmitError("");

    if (!validateForm()) {
      return;
    }

    try {
      await login(formData);
    } catch (error) {
      const errorMessage =
        error instanceof Error ? error.message : "Login failed";
      setSubmitError(errorMessage);
    }
  };

  return (
    <div className="min-h-screen flex items-center justify-center bg-background px-4 sm:px-6">
      <div className="w-full max-w-sm">
        {/* Logo + Header */}
        <div className="text-center mb-8">
          <div className="mx-auto w-14 h-14 rounded-[16px] bg-primary flex items-center justify-center shadow-soft-md mb-5">
            <span className="text-white font-bold text-2xl">E</span>
          </div>
          <h1 className="text-[28px] font-bold tracking-tight text-foreground mb-1">
            Sign In
          </h1>
          <p className="text-[15px] text-muted-foreground">
            Enter your credentials to continue
          </p>
        </div>

        {/* Form Card */}
        <div className="bg-card rounded-2xl border-[0.5px] border-border/60 shadow-soft p-5 sm:p-6">
          <form onSubmit={handleSubmit} className="space-y-4">
            {/* Email Field */}
            <div>
              <Label htmlFor="email">Email Address</Label>
              <Input
                id="email"
                name="email"
                type="email"
                value={formData.email}
                onChange={handleChange}
                placeholder="you@example.com"
                className={errors.email ? "ring-2 ring-destructive/30" : ""}
                disabled={isLoading}
              />
              {errors.email && (
                <p className="mt-1.5 text-[13px] text-destructive">{errors.email}</p>
              )}
            </div>

            {/* Password Field */}
            <div>
              <Label htmlFor="password">Password</Label>
              <Input
                id="password"
                name="password"
                type="password"
                value={formData.password}
                onChange={handleChange}
                placeholder="Enter your password"
                className={errors.password ? "ring-2 ring-destructive/30" : ""}
                disabled={isLoading}
              />
              {errors.password && (
                <p className="mt-1.5 text-[13px] text-destructive">{errors.password}</p>
              )}
            </div>

            {/* Submit Error */}
            {submitError && (
              <p className="text-[13px] text-destructive text-center py-1">
                {submitError}
              </p>
            )}

            {/* Submit Button */}
            <Button type="submit" className="w-full" size="lg" disabled={isLoading}>
              {isLoading ? "Signing in..." : "Sign In"}
            </Button>
          </form>
        </div>

        {/* Switch to Register */}
        <div className="mt-6 text-center">
          <p className="text-[14px] text-muted-foreground">
            Don't have an account?{" "}
            <button
              type="button"
              onClick={onSwitchToRegister}
              className="font-semibold text-primary hover:text-primary/80 transition-colors"
              disabled={isLoading}
            >
              Sign up
            </button>
          </p>
        </div>
      </div>
    </div>
  );
};

export default Login;
