import React, { useState, useEffect } from "react";
import { Button } from "@/shared/ui/button";
import { Input } from "@/shared/ui/input";
import { Label } from "@/shared/ui/label";
import { Textarea } from "@/shared/ui/textarea";
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@/shared/ui/select";
import { Card, CardContent, CardHeader, CardTitle } from "@/shared/ui/card";
import { Badge } from "@/shared/ui/badge";
import { ArrowLeft, Save, Eye, Edit, Calendar, DollarSign } from "lucide-react";
import {
  OrderType,
  OrderStatus,
  ExternalEntityType,
  OrderFormData,
  OrderDetailResponse,
} from "@/features/orders/types/order";
import {
  createOrder,
  updateOrder,
  getOrderById,
} from "@/features/orders/services/orderService";

const isValidUUID = (uuid: string): boolean => {
  const uuidRegex =
    /^[0-9a-f]{8}-[0-9a-f]{4}-[1-5][0-9a-f]{3}-[89ab][0-9a-f]{3}-[0-9a-f]{12}$/i;
  return uuidRegex.test(uuid);
};

const formatAsUUID = (value: string): string => {
  // Remove any non-alphanumeric characters
  const cleaned = value.replace(/[^a-fA-F0-9]/g, "");

  // If it's already 32 characters, format as UUID
  if (cleaned.length === 32) {
    return `${cleaned.slice(0, 8)}-${cleaned.slice(8, 12)}-${cleaned.slice(
      12,
      16,
    )}-${cleaned.slice(16, 20)}-${cleaned.slice(20, 32)}`;
  }

  // If it's less than 32 characters, pad with zeros
  if (cleaned.length < 32) {
    const padded = cleaned.padEnd(32, "0");
    return `${padded.slice(0, 8)}-${padded.slice(8, 12)}-${padded.slice(
      12,
      16,
    )}-${padded.slice(16, 20)}-${padded.slice(20, 32)}`;
  }

  return value;
};

interface StatusColorsType {
  [key: string]: string;
}

interface OrderFormViewProps {
  orderId?: string;
  orderType?: OrderType;
  mode: "view" | "edit" | "create";
  onSave?: () => void;
  onBack: () => void;
}

type FormState = {
  order_number: string;
  order_type: OrderType;
  external_entity_id: string;
  external_entity_type: ExternalEntityType;
  order_date: string;
  total_amount: number;
  status: OrderStatus;
  created_by_id: string;
  notes: string;
  // Customer Order specific
  customer_reference: string;
  shipping_address: string;
  billing_address: string;
  promised_delivery_date: string;
  payment_method: string;
  shipping_method: string;
  discount_amount: number;
  // Purchase Order specific
  vendor_reference: string;
  expected_delivery_date: string;
  payment_terms: string;
  shipping_terms: string;
  approval_date: string;
  // Distributor Order specific
  territory: string;
  commission_rate: number;
  target_resale_amount: number;
  agreement_reference: string;
  marketing_support: string;
};

const OrderFormView: React.FC<OrderFormViewProps> = ({
  orderId,
  orderType = "customer_order",
  mode,
  onSave,
  onBack,
}) => {
  const [formData, setFormData] = useState<FormState>({
    order_number: "",
    order_type: orderType,
    external_entity_id: "",
    external_entity_type:
      orderType === "customer_order"
        ? "customer"
        : orderType === "purchase_order"
          ? "vendor"
          : "distributor",
    order_date: new Date().toISOString().split("T")[0],
    total_amount: 0,
    status: "draft",
    created_by_id: "",
    notes: "",
    // Customer Order specific
    customer_reference: "",
    shipping_address: "",
    billing_address: "",
    promised_delivery_date: "",
    payment_method: "",
    shipping_method: "",
    discount_amount: 0,
    // Purchase Order specific
    vendor_reference: "",
    expected_delivery_date: "",
    payment_terms: "",
    shipping_terms: "",
    approval_date: "",
    // Distributor Order specific
    territory: "",
    commission_rate: 0,
    target_resale_amount: 0,
    agreement_reference: "",
    marketing_support: "",
  });

  const [loading, setLoading] = useState<boolean>(false);
  const [error, setError] = useState<string | null>(null);
  const [orderData, setOrderData] = useState<OrderDetailResponse | null>(null);

  useEffect(() => {
    if (orderId && (mode === "view" || mode === "edit")) {
      fetchOrderData();
    }
  }, [orderId, mode]);

  const fetchOrderData = async () => {
    if (!orderId) return;

    try {
      setLoading(true);
      setError(null);
      const data = await getOrderById(orderId);
      setOrderData(data);

      // Populate form data
      setFormData({
        order_number: data.order_number || "",
        order_type: data.order_type,
        external_entity_id: data.external_entity_id || "",
        external_entity_type: data.external_entity_type,
        order_date: data.order_date ? data.order_date.split("T")[0] : "",
        total_amount: data.total_amount || 0,
        status: data.status,
        created_by_id: data.created_by_id || "",
        notes: data.notes || "",
        // Customer Order specific
        customer_reference: data.customer_order?.customer_reference || "",
        shipping_address: data.customer_order?.shipping_address || "",
        billing_address: data.customer_order?.billing_address || "",
        promised_delivery_date: data.customer_order?.promised_delivery_date
          ? data.customer_order.promised_delivery_date.split("T")[0]
          : "",
        payment_method: data.customer_order?.payment_method || "",
        shipping_method: data.customer_order?.shipping_method || "",
        discount_amount: data.customer_order?.discount_amount || 0,
        // Purchase Order specific
        vendor_reference: data.purchase_order?.vendor_reference || "",
        expected_delivery_date: data.purchase_order?.expected_delivery_date
          ? data.purchase_order.expected_delivery_date.split("T")[0]
          : "",
        payment_terms: data.purchase_order?.payment_terms || "",
        shipping_terms: data.purchase_order?.shipping_terms || "",
        approval_date: data.purchase_order?.approval_date
          ? data.purchase_order.approval_date.split("T")[0]
          : "",
        // Distributor Order specific
        territory: data.distributor_order?.territory || "",
        commission_rate: data.distributor_order?.commission_rate || 0,
        target_resale_amount: data.distributor_order?.target_resale_amount || 0,
        agreement_reference: data.distributor_order?.agreement_reference || "",
        marketing_support: data.distributor_order?.marketing_support || "",
      });
    } catch (err) {
      console.error("Error fetching order data:", err);
      setError("Failed to load order data. Please try again.");
    } finally {
      setLoading(false);
    }
  };

  const handleChange = (
    e: React.ChangeEvent<HTMLInputElement | HTMLTextAreaElement>,
  ): void => {
    const { name, value, type } = e.target;

    setFormData((prev) => ({
      ...prev,
      [name]: type === "number" ? parseFloat(value) || 0 : value,
    }));
  };

  const handleSelectChange = (name: string, value: string): void => {
    setFormData((prev) => ({
      ...prev,
      [name]: value,
    }));
  };

  const handleSubmit = async (
    e: React.FormEvent<HTMLFormElement>,
  ): Promise<void> => {
    e.preventDefault();
    setLoading(true);
    setError(null);

    try {
      // Validate required fields
      if (!formData.order_number.trim()) {
        throw new Error("Order number is required");
      }

      if (!formData.external_entity_id.trim()) {
        throw new Error("External entity ID is required");
      }

      if (!formData.created_by_id.trim()) {
        throw new Error("Created by ID is required");
      }

      // Format UUIDs
      const external_entity_id = isValidUUID(formData.external_entity_id)
        ? formData.external_entity_id
        : formatAsUUID(formData.external_entity_id);

      const created_by_id = isValidUUID(formData.created_by_id)
        ? formData.created_by_id
        : formatAsUUID(formData.created_by_id);

      // Prepare order data
      const orderData: OrderFormData = {
        order_number: formData.order_number.trim(),
        order_type: formData.order_type,
        external_entity_id,
        external_entity_type: formData.external_entity_type,
        order_date: formData.order_date,
        total_amount: formData.total_amount,
        status: formData.status,
        created_by_id,
        notes: formData.notes.trim() || undefined,
      };

      // Add type-specific fields
      if (formData.order_type === "customer_order") {
        orderData.customer_reference = formData.customer_reference || undefined;
        orderData.shipping_address = formData.shipping_address || undefined;
        orderData.billing_address = formData.billing_address || undefined;
        orderData.promised_delivery_date =
          formData.promised_delivery_date || undefined;
        orderData.payment_method = formData.payment_method || undefined;
        orderData.shipping_method = formData.shipping_method || undefined;
        orderData.discount_amount = formData.discount_amount;
      } else if (formData.order_type === "purchase_order") {
        orderData.vendor_reference = formData.vendor_reference || undefined;
        orderData.expected_delivery_date =
          formData.expected_delivery_date || undefined;
        orderData.payment_terms = formData.payment_terms || undefined;
        orderData.shipping_terms = formData.shipping_terms || undefined;
        orderData.approval_date = formData.approval_date || undefined;
      } else if (formData.order_type === "distributor_order") {
        orderData.territory = formData.territory || undefined;
        orderData.commission_rate = formData.commission_rate;
        orderData.target_resale_amount = formData.target_resale_amount;
        orderData.agreement_reference =
          formData.agreement_reference || undefined;
        orderData.marketing_support = formData.marketing_support || undefined;
      }

      if (mode === "create") {
        await createOrder(orderData);
      } else if (mode === "edit" && orderId) {
        await updateOrder(orderId, orderData);
      }

      onSave?.();
    } catch (err: any) {
      console.error("Error saving order:", err);
      setError(err.message || "Failed to save order. Please try again.");
    } finally {
      setLoading(false);
    }
  };

  const getTitle = () => {
    const orderTypeLabel =
      formData.order_type === "customer_order"
        ? "Customer Order"
        : formData.order_type === "purchase_order"
          ? "Purchase Order"
          : "Distributor Order";

    if (mode === "create") return `Create ${orderTypeLabel}`;
    if (mode === "edit") return `Edit ${orderTypeLabel}`;
    return `View ${orderTypeLabel}`;
  };

  const statusColors: StatusColorsType = {
    draft: "bg-gray-100 text-gray-800",
    submitted: "bg-blue-100 text-blue-800",
    approved: "bg-green-100 text-green-800",
    fulfilled: "bg-purple-100 text-purple-800",
    partially_fulfilled: "bg-yellow-100 text-yellow-800",
    cancelled: "bg-red-100 text-red-800",
    paid: "bg-emerald-100 text-emerald-800",
  };

  if (loading && mode !== "create") {
    return (
      <div className="flex justify-center items-center h-64">
        <div className="text-lg">Loading order data...</div>
      </div>
    );
  }

  return (
    <div className="max-w-4xl mx-auto space-y-6">
      {/* Header */}
      <div className="flex items-center justify-between">
        <div className="flex items-center space-x-4">
          <Button variant="outline" onClick={onBack} size="sm">
            <ArrowLeft className="h-4 w-4 mr-2" />
            Back
          </Button>
          <div>
            <h1 className="text-3xl font-bold text-gray-900">{getTitle()}</h1>
            {mode === "view" && orderData && (
              <div className="flex items-center space-x-2 mt-1">
                <Badge className={`${statusColors[orderData.status]} text-xs`}>
                  {orderData.status.replace("_", " ").toUpperCase()}
                </Badge>
                <span className="text-gray-500">•</span>
                <span className="text-gray-600">
                  Created{" "}
                  {new Date(orderData.created_at || "").toLocaleDateString()}
                </span>
              </div>
            )}
          </div>
        </div>
        {mode === "view" && (
          <div className="flex space-x-2">
            <Button variant="outline" size="sm">
              <Edit className="h-4 w-4 mr-2" />
              Edit
            </Button>
          </div>
        )}
      </div>

      {error && (
        <div className="bg-red-50 border border-red-200 rounded-md p-4">
          <div className="text-red-800">{error}</div>
        </div>
      )}

      <form onSubmit={handleSubmit} className="space-y-6">
        {/* Basic Information */}
        <Card>
          <CardHeader>
            <CardTitle>Basic Information</CardTitle>
          </CardHeader>
          <CardContent className="space-y-4">
            <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
              <div>
                <Label htmlFor="order_number">Order Number *</Label>
                <Input
                  id="order_number"
                  name="order_number"
                  value={formData.order_number}
                  onChange={handleChange}
                  disabled={mode === "view"}
                  required
                />
              </div>
              <div>
                <Label htmlFor="order_type">Order Type *</Label>
                <Select
                  value={formData.order_type}
                  onValueChange={(value) =>
                    handleSelectChange("order_type", value)
                  }
                  disabled={mode === "view" || mode === "edit"}
                >
                  <SelectTrigger>
                    <SelectValue />
                  </SelectTrigger>
                  <SelectContent>
                    <SelectItem value="customer_order">
                      Customer Order
                    </SelectItem>
                    <SelectItem value="purchase_order">
                      Purchase Order
                    </SelectItem>
                    <SelectItem value="distributor_order">
                      Distributor Order
                    </SelectItem>
                  </SelectContent>
                </Select>
              </div>
            </div>

            <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
              <div>
                <Label htmlFor="external_entity_id">External Entity ID *</Label>
                <Input
                  id="external_entity_id"
                  name="external_entity_id"
                  value={formData.external_entity_id}
                  onChange={handleChange}
                  disabled={mode === "view"}
                  placeholder="UUID of customer/vendor/distributor"
                  required
                />
              </div>
              <div>
                <Label htmlFor="created_by_id">Created By ID *</Label>
                <Input
                  id="created_by_id"
                  name="created_by_id"
                  value={formData.created_by_id}
                  onChange={handleChange}
                  disabled={mode === "view"}
                  placeholder="UUID of user creating the order"
                  required
                />
              </div>
            </div>

            <div className="grid grid-cols-1 md:grid-cols-3 gap-4">
              <div>
                <Label htmlFor="order_date">Order Date *</Label>
                <Input
                  id="order_date"
                  name="order_date"
                  type="date"
                  value={formData.order_date}
                  onChange={handleChange}
                  disabled={mode === "view"}
                  required
                />
              </div>
              <div>
                <Label htmlFor="total_amount">Total Amount</Label>
                <Input
                  id="total_amount"
                  name="total_amount"
                  type="number"
                  step="0.01"
                  value={formData.total_amount}
                  onChange={handleChange}
                  disabled={mode === "view"}
                />
              </div>
              <div>
                <Label htmlFor="status">Status</Label>
                <Select
                  value={formData.status}
                  onValueChange={(value) => handleSelectChange("status", value)}
                  disabled={mode === "view"}
                >
                  <SelectTrigger>
                    <SelectValue />
                  </SelectTrigger>
                  <SelectContent>
                    <SelectItem value="draft">Draft</SelectItem>
                    <SelectItem value="submitted">Submitted</SelectItem>
                    <SelectItem value="approved">Approved</SelectItem>
                    <SelectItem value="fulfilled">Fulfilled</SelectItem>
                    <SelectItem value="partially_fulfilled">
                      Partially Fulfilled
                    </SelectItem>
                    <SelectItem value="cancelled">Cancelled</SelectItem>
                    <SelectItem value="paid">Paid</SelectItem>
                  </SelectContent>
                </Select>
              </div>
            </div>

            <div>
              <Label htmlFor="notes">Notes</Label>
              <Textarea
                id="notes"
                name="notes"
                value={formData.notes}
                onChange={handleChange}
                disabled={mode === "view"}
                rows={3}
              />
            </div>
          </CardContent>
        </Card>

        {/* Order Type Specific Fields */}
        {formData.order_type === "customer_order" && (
          <Card>
            <CardHeader>
              <CardTitle>Customer Order Details</CardTitle>
            </CardHeader>
            <CardContent className="space-y-4">
              <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
                <div>
                  <Label htmlFor="customer_reference">Customer Reference</Label>
                  <Input
                    id="customer_reference"
                    name="customer_reference"
                    value={formData.customer_reference}
                    onChange={handleChange}
                    disabled={mode === "view"}
                  />
                </div>
                <div>
                  <Label htmlFor="promised_delivery_date">
                    Promised Delivery Date
                  </Label>
                  <Input
                    id="promised_delivery_date"
                    name="promised_delivery_date"
                    type="date"
                    value={formData.promised_delivery_date}
                    onChange={handleChange}
                    disabled={mode === "view"}
                  />
                </div>
              </div>

              <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
                <div>
                  <Label htmlFor="payment_method">Payment Method</Label>
                  <Input
                    id="payment_method"
                    name="payment_method"
                    value={formData.payment_method}
                    onChange={handleChange}
                    disabled={mode === "view"}
                  />
                </div>
                <div>
                  <Label htmlFor="shipping_method">Shipping Method</Label>
                  <Input
                    id="shipping_method"
                    name="shipping_method"
                    value={formData.shipping_method}
                    onChange={handleChange}
                    disabled={mode === "view"}
                  />
                </div>
              </div>

              <div>
                <Label htmlFor="discount_amount">Discount Amount</Label>
                <Input
                  id="discount_amount"
                  name="discount_amount"
                  type="number"
                  step="0.01"
                  value={formData.discount_amount}
                  onChange={handleChange}
                  disabled={mode === "view"}
                />
              </div>

              <div>
                <Label htmlFor="shipping_address">Shipping Address</Label>
                <Textarea
                  id="shipping_address"
                  name="shipping_address"
                  value={formData.shipping_address}
                  onChange={handleChange}
                  disabled={mode === "view"}
                  rows={2}
                />
              </div>

              <div>
                <Label htmlFor="billing_address">Billing Address</Label>
                <Textarea
                  id="billing_address"
                  name="billing_address"
                  value={formData.billing_address}
                  onChange={handleChange}
                  disabled={mode === "view"}
                  rows={2}
                />
              </div>
            </CardContent>
          </Card>
        )}

        {formData.order_type === "purchase_order" && (
          <Card>
            <CardHeader>
              <CardTitle>Purchase Order Details</CardTitle>
            </CardHeader>
            <CardContent className="space-y-4">
              <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
                <div>
                  <Label htmlFor="vendor_reference">Vendor Reference</Label>
                  <Input
                    id="vendor_reference"
                    name="vendor_reference"
                    value={formData.vendor_reference}
                    onChange={handleChange}
                    disabled={mode === "view"}
                  />
                </div>
                <div>
                  <Label htmlFor="expected_delivery_date">
                    Expected Delivery Date
                  </Label>
                  <Input
                    id="expected_delivery_date"
                    name="expected_delivery_date"
                    type="date"
                    value={formData.expected_delivery_date}
                    onChange={handleChange}
                    disabled={mode === "view"}
                  />
                </div>
              </div>

              <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
                <div>
                  <Label htmlFor="payment_terms">Payment Terms</Label>
                  <Input
                    id="payment_terms"
                    name="payment_terms"
                    value={formData.payment_terms}
                    onChange={handleChange}
                    disabled={mode === "view"}
                  />
                </div>
                <div>
                  <Label htmlFor="shipping_terms">Shipping Terms</Label>
                  <Input
                    id="shipping_terms"
                    name="shipping_terms"
                    value={formData.shipping_terms}
                    onChange={handleChange}
                    disabled={mode === "view"}
                  />
                </div>
              </div>

              <div>
                <Label htmlFor="approval_date">Approval Date</Label>
                <Input
                  id="approval_date"
                  name="approval_date"
                  type="date"
                  value={formData.approval_date}
                  onChange={handleChange}
                  disabled={mode === "view"}
                />
              </div>
            </CardContent>
          </Card>
        )}

        {formData.order_type === "distributor_order" && (
          <Card>
            <CardHeader>
              <CardTitle>Distributor Order Details</CardTitle>
            </CardHeader>
            <CardContent className="space-y-4">
              <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
                <div>
                  <Label htmlFor="territory">Territory</Label>
                  <Input
                    id="territory"
                    name="territory"
                    value={formData.territory}
                    onChange={handleChange}
                    disabled={mode === "view"}
                  />
                </div>
                <div>
                  <Label htmlFor="agreement_reference">
                    Agreement Reference
                  </Label>
                  <Input
                    id="agreement_reference"
                    name="agreement_reference"
                    value={formData.agreement_reference}
                    onChange={handleChange}
                    disabled={mode === "view"}
                  />
                </div>
              </div>

              <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
                <div>
                  <Label htmlFor="commission_rate">Commission Rate (%)</Label>
                  <Input
                    id="commission_rate"
                    name="commission_rate"
                    type="number"
                    step="0.01"
                    value={formData.commission_rate}
                    onChange={handleChange}
                    disabled={mode === "view"}
                  />
                </div>
                <div>
                  <Label htmlFor="target_resale_amount">
                    Target Resale Amount
                  </Label>
                  <Input
                    id="target_resale_amount"
                    name="target_resale_amount"
                    type="number"
                    step="0.01"
                    value={formData.target_resale_amount}
                    onChange={handleChange}
                    disabled={mode === "view"}
                  />
                </div>
              </div>

              <div>
                <Label htmlFor="marketing_support">Marketing Support</Label>
                <Textarea
                  id="marketing_support"
                  name="marketing_support"
                  value={formData.marketing_support}
                  onChange={handleChange}
                  disabled={mode === "view"}
                  rows={3}
                />
              </div>
            </CardContent>
          </Card>
        )}

        {/* Action Buttons */}
        {mode !== "view" && (
          <div className="flex justify-end space-x-4">
            <Button type="button" variant="outline" onClick={onBack}>
              Cancel
            </Button>
            <Button type="submit" disabled={loading}>
              {loading ? (
                "Saving..."
              ) : (
                <>
                  <Save className="h-4 w-4 mr-2" />
                  {mode === "create" ? "Create Order" : "Update Order"}
                </>
              )}
            </Button>
          </div>
        )}
      </form>
    </div>
  );
};

export default OrderFormView;
