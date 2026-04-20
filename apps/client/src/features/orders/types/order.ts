// Order status options
export type OrderStatus =
  | "draft"
  | "submitted"
  | "approved"
  | "fulfilled"
  | "partially_fulfilled"
  | "cancelled"
  | "paid";

// Order types
export type OrderType =
  | "customer_order"
  | "distributor_order"
  | "purchase_order";

// External entity types
export type ExternalEntityType = "customer" | "distributor" | "vendor";

// Base Order Interface
export interface Order {
  id: string;
  tenant_id?: string;
  order_number: string;
  order_type: OrderType;
  external_entity_id: string;
  external_entity_type: ExternalEntityType;
  order_date: string;
  total_amount: number;
  status: OrderStatus;
  created_by_id: string;
  notes?: string;
  meta_info?: Record<string, any>;
  created_at?: string;
  updated_at?: string;
}

// Order Item Interface
export interface OrderItem {
  id: string;
  item_id: string;
  quantity: number;
  unit_price: number;
  extended_price: number;
  notes?: string;
}

// Customer Order Interface
export interface CustomerOrderDetails {
  customer_reference?: string;
  shipping_address?: string;
  billing_address?: string;
  promised_delivery_date?: string;
  payment_method?: string;
  shipping_method?: string;
  discount_amount?: number;
}

export interface CustomerOrder extends Order {
  customer_order: CustomerOrderDetails;
}

// Purchase Order Interface
export interface PurchaseOrderDetails {
  vendor_reference?: string;
  expected_delivery_date?: string;
  payment_terms?: string;
  shipping_terms?: string;
  approval_date?: string;
}

export interface PurchaseOrder extends Order {
  purchase_order: PurchaseOrderDetails;
}

// Distributor Order Interface
export interface DistributorOrderDetails {
  territory?: string;
  commission_rate?: number;
  target_resale_amount?: number;
  agreement_reference?: string;
  marketing_support?: string;
}

export interface DistributorOrder extends Order {
  distributor_order: DistributorOrderDetails;
}

// Order History Interface
export interface OrderHistory {
  id: string;
  order_id: string;
  user_id?: string;
  action: string;
  previous_status?: OrderStatus;
  new_status?: OrderStatus;
  notes?: string;
  created_at?: string;
}

// Order Form Interface for creating/updating orders
export interface OrderFormData {
  order_number: string;
  order_type: OrderType;
  external_entity_id: string;
  external_entity_type: ExternalEntityType;
  order_date: string;
  total_amount?: number;
  status: OrderStatus;
  created_by_id: string;
  notes?: string;
  meta_info?: string; // JSON string

  // Customer Order specific
  customer_reference?: string;
  shipping_address?: string;
  billing_address?: string;
  promised_delivery_date?: string;
  payment_method?: string;
  shipping_method?: string;
  discount_amount?: number;

  // Purchase Order specific
  vendor_reference?: string;
  expected_delivery_date?: string;
  payment_terms?: string;
  shipping_terms?: string;
  approval_date?: string;

  // Distributor Order specific
  territory?: string;
  commission_rate?: number;
  target_resale_amount?: number;
  agreement_reference?: string;
  marketing_support?: string;

  // Order items
  items?: string; // JSON string
}

// API Response Interfaces
export interface OrderListResponse extends Array<Order> {}

export interface OrderDetailResponse extends Order {
  customer_order?: CustomerOrderDetails;
  purchase_order?: PurchaseOrderDetails;
  distributor_order?: DistributorOrderDetails;
  items?: OrderItem[];
  history?: OrderHistory[];
}

export interface OrderCreateResponse {
  id: string;
}

export interface OrderUpdateResponse {
  id: string;
  message: string;
}

export interface OrderHistoryCreateResponse {
  id: string;
  order_id: string;
  action: string;
  created_at?: string;
}

export interface OrderHistoryListResponse extends Array<OrderHistory> {}
