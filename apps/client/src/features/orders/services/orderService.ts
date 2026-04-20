import axios from "axios";
import {
  Order,
  OrderDetailResponse,
  OrderCreateResponse,
  OrderUpdateResponse,
  OrderHistory,
  OrderHistoryCreateResponse,
  OrderFormData,
} from "@/features/orders/types/order";

const API_URL = "/api/v1";

// Set up axios interceptor to include tenant ID and authorization token in all requests
axios.interceptors.request.use((config) => {
  // Get tenant ID from local storage
  const tenantId = localStorage.getItem("tenantId");
  if (tenantId) {
    config.headers["X-Tenant-ID"] = tenantId;
  }

  // Get authorization token from local storage
  const accessToken = localStorage.getItem("accessToken");
  if (accessToken) {
    config.headers["Authorization"] = `Bearer ${accessToken}`;
  }

  return config;
});

// Generic Order API methods
export const getAllOrders = async (): Promise<Order[]> => {
  try {
    const response = await axios.get(`${API_URL}/order`);
    return response.data;
  } catch (error) {
    console.error("Error fetching orders:", error);
    throw error;
  }
};

export const getOrderById = async (
  orderId: string,
): Promise<OrderDetailResponse> => {
  try {
    const response = await axios.get(`${API_URL}/order/${orderId}`);
    return response.data;
  } catch (error) {
    console.error("Error fetching order:", error);
    throw error;
  }
};

export const createOrder = async (
  orderData: OrderFormData,
): Promise<OrderCreateResponse> => {
  try {
    const response = await axios.post(`${API_URL}/order`, orderData);
    return response.data;
  } catch (error) {
    console.error("Error creating order:", error);
    throw error;
  }
};

export const updateOrder = async (
  orderId: string,
  orderData: Partial<OrderFormData>,
): Promise<OrderUpdateResponse> => {
  try {
    const response = await axios.put(`${API_URL}/order/${orderId}`, orderData);
    return response.data;
  } catch (error) {
    console.error("Error updating order:", error);
    throw error;
  }
};

export const deleteOrder = async (orderId: string): Promise<boolean> => {
  try {
    await axios.delete(`${API_URL}/order/${orderId}`);
    return true;
  } catch (error) {
    console.error("Error deleting order:", error);
    throw error;
  }
};

// Customer Order specific methods
export const getCustomerOrders = async (): Promise<Order[]> => {
  try {
    const response = await axios.get(`${API_URL}/order/customer`);
    return response.data;
  } catch (error) {
    console.error("Error fetching customer orders:", error);
    throw error;
  }
};

export const getCustomerOrderById = async (
  orderId: string,
): Promise<OrderDetailResponse> => {
  try {
    const response = await axios.get(`${API_URL}/order/customer/${orderId}`);
    return response.data;
  } catch (error) {
    console.error("Error fetching customer order:", error);
    throw error;
  }
};

// Purchase Order specific methods
export const getPurchaseOrders = async (): Promise<Order[]> => {
  try {
    const response = await axios.get(`${API_URL}/order/purchase`);
    return response.data;
  } catch (error) {
    console.error("Error fetching purchase orders:", error);
    throw error;
  }
};

export const getPurchaseOrderById = async (
  orderId: string,
): Promise<OrderDetailResponse> => {
  try {
    const response = await axios.get(`${API_URL}/order/purchase/${orderId}`);
    return response.data;
  } catch (error) {
    console.error("Error fetching purchase order:", error);
    throw error;
  }
};

// Distributor Order specific methods
export const getDistributorOrders = async (): Promise<Order[]> => {
  try {
    const response = await axios.get(`${API_URL}/order/distributor`);
    return response.data;
  } catch (error) {
    console.error("Error fetching distributor orders:", error);
    throw error;
  }
};

export const getDistributorOrderById = async (
  orderId: string,
): Promise<OrderDetailResponse> => {
  try {
    const response = await axios.get(`${API_URL}/order/distributor/${orderId}`);
    return response.data;
  } catch (error) {
    console.error("Error fetching distributor order:", error);
    throw error;
  }
};

// Order History methods
export const getOrderHistory = async (
  orderId: string,
): Promise<OrderHistory[]> => {
  try {
    const response = await axios.get(`${API_URL}/order/${orderId}/history`);
    return response.data;
  } catch (error) {
    console.error("Error fetching order history:", error);
    throw error;
  }
};

export const addOrderHistoryEntry = async (
  orderId: string,
  historyData: any,
): Promise<OrderHistoryCreateResponse> => {
  try {
    const response = await axios.post(
      `${API_URL}/order/${orderId}/history`,
      historyData,
    );
    return response.data;
  } catch (error) {
    console.error("Error adding order history entry:", error);
    throw error;
  }
};

// Order Items methods
export const getOrderItems = async (orderId: string): Promise<any[]> => {
  try {
    const response = await axios.get(`${API_URL}/order/${orderId}/items`);
    return response.data;
  } catch (error) {
    console.error("Error fetching order items:", error);
    throw error;
  }
};

export const addOrderItem = async (
  orderId: string,
  itemData: any,
): Promise<any> => {
  try {
    const response = await axios.post(
      `${API_URL}/order/${orderId}/items`,
      itemData,
    );
    return response.data;
  } catch (error) {
    console.error("Error adding order item:", error);
    throw error;
  }
};

export const updateOrderItem = async (
  orderId: string,
  itemId: string,
  itemData: any,
): Promise<any> => {
  try {
    const response = await axios.put(
      `${API_URL}/order/${orderId}/items/${itemId}`,
      itemData,
    );
    return response.data;
  } catch (error) {
    console.error("Error updating order item:", error);
    throw error;
  }
};

export const deleteOrderItem = async (
  orderId: string,
  itemId: string,
): Promise<boolean> => {
  try {
    await axios.delete(`${API_URL}/order/${orderId}/items/${itemId}`);
    return true;
  } catch (error) {
    console.error("Error deleting order item:", error);
    throw error;
  }
};
